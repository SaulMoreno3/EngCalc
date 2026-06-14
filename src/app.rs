use crate::core::env::Environment;
use crate::core::formatter;
use crate::storage::history::History;
use crate::tui::events::Action;
use crate::tui::input::InputBuffer;
use std::collections::HashMap;

fn char_to_byte_index(input: &str, char_index: usize) -> usize {
    input
        .char_indices()
        .nth(char_index)
        .map(|(byte_index, _)| byte_index)
        .unwrap_or(input.len())
}

pub struct App {
    pub input: InputBuffer,
    pub env: Environment,
    pub user_vars: HashMap<String, crate::core::value::Value>,
    pub history: History,
    pub last_result: Option<String>,
    pub last_error: Option<String>,
    pub history_index: Option<usize>,
    pub running: bool,
    pub is_command_mode: bool,
    pub show_consts: bool,
    pub show_help: bool,
    pub show_functions: bool,
    pub consts_search: String,
    pub consts_selected: usize,
    pub funcs_search: String,
    pub funcs_selected: usize,
    // Autocomplete/help popup state
    pub show_autocomplete: bool,
    pub autocomplete_suggestions: Vec<String>,
    pub autocomplete_selected: usize,
    // Signature help state (shows when typing function arguments)
    pub show_signature_help: bool,
    pub signature_help_func: Option<String>,
    pub signature_help_param_index: usize,
}

impl App {
    pub fn new() -> Self {
        let history = History::load().unwrap_or_default();
        let mut env = Environment::new();
        let mut user_vars = HashMap::new();

        // Restore the last workspace from history if available
        if let Some(last_entry) = history.entries.last() {
            // Restore variables
            for (name, value) in &last_entry.workspace.variables {
                let val = value.to_value();
                env.set(name.clone(), val.clone());
                user_vars.insert(name.clone(), val);
            }

            // Restore functions
            use crate::core::env::UserFunction;
            use crate::core::parser;
            for (_name, func_def) in &last_entry.workspace.functions {
                if let Ok(body_expr) = parser::parse(&func_def.body) {
                    let func = UserFunction {
                        name: func_def.name.clone(),
                        params: func_def.params.clone(),
                        body: body_expr,
                    };
                    env.set_function(func);
                }
            }
        }

        Self {
            input: InputBuffer::new(),
            env,
            user_vars,
            history,
            last_result: None,
            last_error: None,
            history_index: None,
            running: true,
            is_command_mode: false,
            show_consts: false,
            show_help: false,
            show_functions: false,
            consts_search: String::new(),
            consts_selected: 0,
            funcs_search: String::new(),
            funcs_selected: 0,
            show_autocomplete: false,
            autocomplete_suggestions: Vec::new(),
            autocomplete_selected: 0,
            show_signature_help: false,
            signature_help_func: None,
            signature_help_param_index: 0,
        }
    }

    /// Update signature help based on current input (detect function calls)
    pub fn update_signature_help(&mut self) {
        let content = self.input.content();
        let cursor = self.input.cursor_pos();
        let cursor_byte = char_to_byte_index(&content, cursor);
        let before_cursor = &content[..cursor_byte];

        // Scan backwards from cursor, tracking paren depth.
        // We want the '(' that is NOT closed before the cursor (depth 0 -> -1).
        let mut depth: i32 = 0;
        let mut paren_pos: Option<usize> = None;

        for (pos, ch) in before_cursor.char_indices().rev() {
            match ch {
                ')' | ']' | '}' => depth += 1,
                '(' | '[' | '{' => {
                    depth -= 1;
                    if depth < 0 {
                        paren_pos = Some(pos);
                        break;
                    }
                }
                _ => {}
            }
        }

        if let Some(paren_pos) = paren_pos {
            let before_paren = &content[..paren_pos];

            // Find the start of the function name
            let name_start = before_paren
                .rfind(|c: char| !c.is_alphanumeric() && c != '_')
                .map(|i| i + 1)
                .unwrap_or(0);

            let func_name = &before_paren[name_start..];

            if !func_name.is_empty() {
                // Check if this is a known function
                if let Some(func_info) = crate::core::functions::get_function_info(func_name) {
                    // Count which parameter we're on by counting commas at depth 0
                    // after the target paren, up to the cursor.
                    // But we must skip commas inside nested parens.
                    let after_paren = &content[paren_pos + 1..cursor_byte];
                    let mut param_index = 0;
                    let mut inner_depth: i32 = 0;
                    for ch in after_paren.chars() {
                        match ch {
                            '(' | '[' | '{' => inner_depth += 1,
                            ')' | ']' | '}' => inner_depth -= 1,
                            ',' if inner_depth == 0 => param_index += 1,
                            _ => {}
                        }
                    }

                    self.show_signature_help = true;
                    self.signature_help_func = Some(func_info.name.to_string());
                    self.signature_help_param_index = param_index;
                    return;
                }
            }
        }

        // No function call detected
        self.show_signature_help = false;
        self.signature_help_func = None;
    }

    /// Update autocomplete suggestions based on current input
    pub fn update_autocomplete(&mut self) {
        let content = self.input.content();
        let cursor = self.input.cursor_pos();
        let cursor_byte = char_to_byte_index(&content, cursor);
        
        // Get the word being typed (from last space or start to cursor)
        let before_cursor = &content[..cursor_byte];
        let word_start = before_cursor.rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| i + 1)
            .unwrap_or(0);
        let current_word = &before_cursor[word_start..];
        
        if current_word.len() >= 1 {
            let mut suggestions = Vec::new();
            let word_lower = current_word.to_lowercase();
            
            // Match against built-in functions - search for contains, prioritize starts_with
            for func in crate::core::functions::list_functions() {
                let func_name_lower = func.name.to_lowercase();
                if func_name_lower.contains(&word_lower) {
                    let priority = if func_name_lower.starts_with(&word_lower) { 0 } else { 1 };
                    suggestions.push((priority, func.name.to_string(), format!("{}({})", func.name, func.params), func.description.to_string()));
                }
            }
            
            // Match against user-defined functions
            for (name, func) in self.env.iter_functions() {
                let name_lower = name.to_lowercase();
                if name_lower.contains(&word_lower) {
                    let priority = if name_lower.starts_with(&word_lower) { 0 } else { 1 };
                    suggestions.push((priority, name.clone(), format!("{}({})", name, func.params.join(", ")), "User-defined function".to_string()));
                }
            }
            
            // Match against variables
            for (name, value) in &self.user_vars {
                let name_lower = name.to_lowercase();
                if name_lower.contains(&word_lower) {
                    let priority = if name_lower.starts_with(&word_lower) { 0 } else { 1 };
                    let value_str = crate::core::formatter::format_value(value);
                    suggestions.push((priority, name.clone(), name.clone(), format!("Variable = {}", value_str)));
                }
            }
            
            // Match against constants
            for c in crate::core::constants::list() {
                let const_name_lower = c.name.to_lowercase();
                if const_name_lower.contains(&word_lower) {
                    let priority = if const_name_lower.starts_with(&word_lower) { 0 } else { 1 };
                    suggestions.push((priority, c.name.to_string(), c.name.to_string(), format!("Constant = {} {}", c.value, c.units)));
                }
            }
            
            // Sort by priority first (starts_with comes before contains), then alphabetically
            suggestions.sort_by(|a, b| {
                match a.0.cmp(&b.0) {
                    std::cmp::Ordering::Equal => a.1.cmp(&b.1),
                    other => other,
                }
            });
            
            // Convert to display strings
            let display_suggestions: Vec<String> = suggestions.into_iter()
                .map(|(_priority, _name, signature, description)| format!("{}|{}", signature, description))
                .collect();
            
            if !display_suggestions.is_empty() {
                self.autocomplete_suggestions = display_suggestions;
                self.show_autocomplete = true;
                self.autocomplete_selected = 0;
            } else {
                // Hide popup when no suggestions match
                self.show_autocomplete = false;
                self.autocomplete_suggestions.clear();
            }
        } else {
            // Hide popup when word is too short or input is empty
            self.show_autocomplete = false;
            self.autocomplete_suggestions.clear();
        }
    }

    /// Accept the currently selected autocomplete suggestion
    pub fn accept_autocomplete(&mut self) {
        if !self.show_autocomplete || self.autocomplete_suggestions.is_empty() {
            return;
        }
        
        let content = self.input.content();
        let cursor = self.input.cursor_pos();
        let cursor_byte = char_to_byte_index(&content, cursor);
        let before_cursor = &content[..cursor_byte];
        let word_start = before_cursor.rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| i + 1)
            .unwrap_or(0);
        
        // Extract just the signature part (before '|')
        let full_suggestion = &self.autocomplete_suggestions[self.autocomplete_selected];
        let signature = full_suggestion.split('|').next().unwrap_or(full_suggestion);
        
        // Replace the current word with the suggestion
        let new_content = format!("{}{}{}", 
            &content[..word_start],
            signature,
            &content[cursor_byte..]
        );
        
        let new_cursor = content[..word_start].chars().count() + signature.chars().count();
        self.input.set_content(new_content);
        self.input.set_cursor_pos(new_cursor);
        self.show_autocomplete = false;
        // Check if we just completed a function name, prepare for signature help
        self.update_signature_help();
    }
    
    /// Navigate autocomplete suggestions
    pub fn autocomplete_next(&mut self) {
        if !self.autocomplete_suggestions.is_empty() {
            self.autocomplete_selected = (self.autocomplete_selected + 1) % self.autocomplete_suggestions.len();
        }
    }
    
    pub fn autocomplete_prev(&mut self) {
        if !self.autocomplete_suggestions.is_empty() {
            let len = self.autocomplete_suggestions.len();
            self.autocomplete_selected = (self.autocomplete_selected + len - 1) % len;
        }
    }

    pub fn handle_action(&mut self, action: Action) {
        if matches!(action, Action::ShowHelp) {
            self.show_help = !self.show_help;
            self.show_consts = false;
            return;
        }

        if matches!(action, Action::ShowConsts) {
            self.show_consts = !self.show_consts;
            self.show_help = false;
            self.show_functions = false;
            if self.show_consts {
                self.consts_search.clear();
                self.consts_selected = 0;
            }
            return;
        }

        if matches!(action, Action::ShowFunctions) {
            self.show_functions = !self.show_functions;
            self.show_help = false;
            self.show_consts = false;
            if self.show_functions {
                self.funcs_search.clear();
                self.funcs_selected = 0;
            }
            return;
        }

        if self.show_functions {
            self.handle_funcs_action(action);
            return;
        }

        if self.show_consts || self.show_functions {
            self.handle_consts_action(action);
            return;
        }

        if self.show_help {
            if matches!(action, Action::Quit) {
                self.running = false;
            }
            self.show_help = false;
            return;
        }

        match action {
            Action::Quit => {
                self.running = false;
            }
            Action::Eval => {
                self.show_signature_help = false;
                self.show_autocomplete = false;
                self.autocomplete_suggestions.clear();
                self.eval_input();
            }
            Action::ClearScreen => {
                self.show_signature_help = false;
                self.clear();
            }
            Action::ClearInput => {
                self.show_signature_help = false;
                self.input.clear();
                self.history_index = None;
            }
            Action::ClearAll => {
                self.history.clear();
                let _ = self.history.save();
                self.user_vars.clear();
                self.env = Environment::new();
                self.input.clear();
                self.history_index = None;
                self.last_result = None;
                self.last_error = None;
            }
            Action::HistoryUp => {
                self.handle_autocomplete_nav(true);
            }
            Action::HistoryDown => {
                self.handle_autocomplete_nav(false);
            }
            Action::Autocomplete => {
                if self.show_autocomplete {
                    self.accept_autocomplete();
                } else {
                    self.autocomplete();
                }
            }
            Action::CursorLeft => {
                self.input.cursor_left();
                self.show_autocomplete = false;
            }
            Action::CursorRight => {
                self.input.cursor_right();
                self.show_autocomplete = false;
            }
            Action::CursorHome => {
                self.input.cursor_home();
                self.show_autocomplete = false;
            }
            Action::CursorEnd => {
                self.input.cursor_end();
                self.show_autocomplete = false;
            }
            Action::DeleteBackward => {
                self.input.delete_char();
                self.update_autocomplete();
                self.update_signature_help();
            }
            Action::DeleteForward => {
                self.input.delete_forward();
                self.update_autocomplete();
                self.update_signature_help();
            }
            Action::InputChar(c) => {
                if c == ':' && self.input.is_empty() {
                    self.is_command_mode = true;
                }
                self.input.insert_char(c);
                self.update_autocomplete();
                self.update_signature_help();
            }
            Action::CommandMode => {
                if !self.input.is_empty() {
                    self.input.clear();
                }
                self.is_command_mode = true;
                self.input.insert_char(':');
            }
            _ => {}
        }
    }

    fn eval_input(&mut self) {
        let expr_str = self.input.content().trim().to_string();

        if expr_str.is_empty() {
            self.is_command_mode = false;
            return;
        }

        if expr_str.starts_with(':') {
            self.handle_command(&expr_str[1..]);
            return;
        }

        match crate::core::parser::parse(&expr_str) {
            Ok(ast) => {
                // Check for function definition: f(x) = expr
                if let Some((name, params, body)) = ast.as_function_def() {
                    use crate::core::env::UserFunction;
                    let func = UserFunction {
                        name: name.to_string(),
                        params: params.to_vec(),
                        body: (*body).clone(),
                    };
                    self.env.set_function(func);
                    let formatted = format!("{}({}) defined", name, params.join(", "));
                    self.last_result = Some(formatted.clone());
                    self.last_error = None;
                    let workspace = self.capture_workspace();
                    self.history
                        .add(expr_str.clone(), formatted, false, workspace);
                } else if let Some((name, val_expr)) = ast.as_assignment() {
                    let name = name.to_string();
                    match val_expr.eval(&self.env) {
                        Ok(value) => {
                            self.env.set(name.clone(), value.clone());
                            self.user_vars.insert(name.clone(), value.clone());
                            // Also store in ans
                            self.env.set("ans".to_string(), value.clone());
                            self.user_vars.insert("ans".to_string(), value.clone());
                            let formatted = formatter::format_assignment(&name, &value);
                            self.last_result = Some(formatted.clone());
                            self.last_error = None;
                            let workspace = self.capture_workspace();
                            self.history
                                .add(expr_str.clone(), formatted, false, workspace);
                        }
                        Err(e) => {
                            let msg = e.to_string();
                            self.last_error = Some(msg.clone());
                            self.last_result = None;
                            let workspace = self.capture_workspace();
                            self.history.add(
                                expr_str,
                                formatter::format_error(&msg),
                                true,
                                workspace,
                            );
                        }
                    }
                } else {
                    match ast.eval(&self.env) {
                        Ok(value) => {
                            // Store result in ans variable
                            self.env.set("ans".to_string(), value.clone());
                            self.user_vars.insert("ans".to_string(), value.clone());
                            let formatted = formatter::format_value(&value);
                            self.last_result = Some(formatted.clone());
                            self.last_error = None;
                            let workspace = self.capture_workspace();
                            self.history
                                .add(expr_str.clone(), formatted, false, workspace);
                        }
                        Err(e) => {
                            let msg = e.to_string();
                            self.last_error = Some(msg.clone());
                            self.last_result = None;
                            let workspace = self.capture_workspace();
                            self.history.add(
                                expr_str,
                                formatter::format_error(&msg),
                                true,
                                workspace,
                            );
                        }
                    }
                }
            }
            Err(e) => {
                let msg = e.to_string();
                self.last_error = Some(msg.clone());
                self.last_result = None;
                let workspace = self.capture_workspace();
                self.history
                    .add(expr_str, formatter::format_error(&msg), true, workspace);
            }
        }

        self.input.clear();
        self.is_command_mode = false;
        self.history_index = None;
        let _ = self.history.save();
    }

    fn handle_command(&mut self, cmd: &str) {
        self.is_command_mode = false;
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        match parts.first() {
            Some(&"help") | Some(&"h") | Some(&"?") => {
                self.show_help = true;
            }
            Some(&"clear") | Some(&"cls") => {
                self.clear();
            }
            Some(&"vars") => {
                let mut msg = String::from("Variables:\n");
                if self.user_vars.is_empty() {
                    msg.push_str("  (none yet)\n");
                } else {
                    for (name, value) in &self.user_vars {
                        msg.push_str(&format!(
                            "  {} = {}\n",
                            name,
                            formatter::format_value(value)
                        ));
                    }
                }
                self.last_result = Some(msg);
                self.last_error = None;
            }
            Some(&"consts") | Some(&"constants") => {
                self.show_consts = true;
            }
            Some(&"history") | Some(&"hist") => {
                let mut msg = String::from("History:\n");
                for entry in self.history.last_n(30) {
                    msg.push_str(&format!("  {} = {}\n", entry.expression, entry.result));
                }
                self.last_result = Some(msg);
                self.last_error = None;
            }
            Some(&"clearhist") => {
                self.history.clear();
                let _ = self.history.save();
                self.last_result = Some("History cleared".to_string());
                self.last_error = None;
            }
            Some(&"quit") | Some(&"exit") | Some(&"q") => {
                self.running = false;
            }
            _ => {
                self.last_error = Some(format!("unknown command: {}", cmd));
                self.last_result = None;
            }
        }

        self.input.clear();
        self.history_index = None;
    }

    fn history_up(&mut self) {
        let exprs = self.history.get_expressions();
        if exprs.is_empty() {
            return;
        }

        match self.history_index {
            None => {
                self.history_index = Some(exprs.len() - 1);
            }
            Some(idx) if idx > 0 => {
                self.history_index = Some(idx - 1);
            }
            Some(_) => {}
        }

        if let Some(idx) = self.history_index {
            if idx < exprs.len() {
                // Clone workspace first to avoid borrow issues
                let workspace = self.history.get_workspace_at(idx).cloned();
                self.input.set_content(exprs[idx].to_string());
                self.input.cursor_end();
                // Restore workspace state
                if let Some(ws) = workspace {
                    self.restore_workspace(&ws);
                }
            }
        }
    }

    fn history_down(&mut self) {
        match self.history_index {
            None => {}
            Some(idx) => {
                let exprs = self.history.get_expressions();
                if idx + 1 >= exprs.len() {
                    self.history_index = None;
                    self.input.clear();
                    self.is_command_mode = false;
                    // Keep current workspace when clearing input
                } else {
                    // Clone workspace first to avoid borrow issues
                    let workspace = self.history.get_workspace_at(idx + 1).cloned();
                    self.history_index = Some(idx + 1);
                    self.input.set_content(exprs[idx + 1].to_string());
                    self.input.cursor_end();
                    // Restore workspace state
                    if let Some(ws) = workspace {
                        self.restore_workspace(&ws);
                    }
                }
            }
        }
    }

    fn autocomplete(&mut self) {
        // New autocomplete system: just trigger the popup
        self.update_autocomplete();
    }
    
    /// Handle autocomplete navigation with arrow keys
    pub fn handle_autocomplete_nav(&mut self, up: bool) {
        if !self.show_autocomplete {
            // If autocomplete not showing, use arrow keys for history
            if up {
                self.history_up();
            } else {
                self.history_down();
            }
        } else {
            // Navigate autocomplete suggestions
            if up {
                self.autocomplete_prev();
            } else {
                self.autocomplete_next();
            }
        }
    }

    pub fn clear(&mut self) {
        self.last_result = None;
        self.last_error = None;
        self.input.clear();
        self.is_command_mode = false;
        self.history_index = None;
    }

    fn handle_consts_action(&mut self, action: Action) {
        match action {
            Action::Quit | Action::ClearInput | Action::ClearScreen => {
                self.show_consts = false;
                self.consts_search.clear();
                self.consts_selected = 0;
            }
            Action::HistoryUp => {
                let filtered = crate::core::constants::search(&self.consts_search);
                if !filtered.is_empty() && self.consts_selected > 0 {
                    self.consts_selected -= 1;
                }
            }
            Action::HistoryDown => {
                let filtered = crate::core::constants::search(&self.consts_search);
                if !filtered.is_empty() && self.consts_selected + 1 < filtered.len() {
                    self.consts_selected += 1;
                }
            }
            Action::Eval | Action::ShowConsts => {
                let filtered = crate::core::constants::search(&self.consts_search);
                if !filtered.is_empty() && self.consts_selected < filtered.len() {
                    let selected = &filtered[self.consts_selected];
                    let insert_at = self.input.cursor_pos();
                    let current = self.input.content();
                    let before: String = current.chars().take(insert_at).collect();
                    let after: String = current.chars().skip(insert_at).collect();
                    self.input
                        .set_content(format!("{}{}{}", before, selected.name, after));
                    self.input.set_cursor_pos(insert_at + selected.name.len());
                }
                self.show_consts = false;
                self.consts_search.clear();
                self.consts_selected = 0;
            }
            Action::InputChar(c) => {
                self.consts_search.push(c);
                self.consts_selected = 0;
            }
            Action::DeleteBackward => {
                self.consts_search.pop();
                self.consts_selected = 0;
            }
            Action::DeleteForward => {
                if !self.consts_search.is_empty() {
                    self.consts_search.remove(self.consts_search.len() - 1);
                    self.consts_selected = 0;
                }
            }
            _ => {}
        }
    }

    fn handle_funcs_action(&mut self, action: Action) {
        match action {
            Action::Quit | Action::ClearInput | Action::ClearScreen => {
                self.show_functions = false;
                self.funcs_search.clear();
                self.funcs_selected = 0;
            }
            Action::HistoryUp => {
                let filtered = self.filtered_functions();
                if !filtered.is_empty() && self.funcs_selected > 0 {
                    self.funcs_selected -= 1;
                }
            }
            Action::HistoryDown => {
                let filtered = self.filtered_functions();
                if !filtered.is_empty() && self.funcs_selected + 1 < filtered.len() {
                    self.funcs_selected += 1;
                }
            }
            Action::Eval | Action::ShowFunctions => {
                let filtered = self.filtered_functions();
                if !filtered.is_empty() && self.funcs_selected < filtered.len() {
                    let selected = &filtered[self.funcs_selected];
                    let insert_text = format!("{}(", selected.name);
                    let insert_at = self.input.cursor_pos();
                    let current = self.input.content();
                    let before: String = current.chars().take(insert_at).collect();
                    let after: String = current.chars().skip(insert_at).collect();
                    self.input
                        .set_content(format!("{}{}{}", before, insert_text, after));
                    self.input.set_cursor_pos(insert_at + insert_text.len());
                }
                self.show_functions = false;
                self.funcs_search.clear();
                self.funcs_selected = 0;
            }
            Action::InputChar(c) => {
                self.funcs_search.push(c);
                self.funcs_selected = 0;
            }
            Action::DeleteBackward => {
                self.funcs_search.pop();
                self.funcs_selected = 0;
            }
            Action::DeleteForward => {
                if !self.funcs_search.is_empty() {
                    self.funcs_search.remove(self.funcs_search.len() - 1);
                    self.funcs_selected = 0;
                }
            }
            _ => {}
        }
    }

    pub fn filtered_functions(&self) -> Vec<crate::core::functions::FunctionInfo> {
        let all = crate::core::functions::list_functions();
        if self.funcs_search.is_empty() {
            return all;
        }
        let q = self.funcs_search.to_lowercase();
        all.into_iter()
            .filter(|f| {
                f.name.to_lowercase().contains(&q) || f.description.to_lowercase().contains(&q)
            })
            .collect()
    }

    /// Capture current workspace state (variables and functions)
    fn capture_workspace(&self) -> crate::storage::history::WorkspaceState {
        use crate::storage::history::{StoredValue, UserFunctionDef, WorkspaceState};

        let mut variables = std::collections::HashMap::new();
        for (name, value) in &self.user_vars {
            variables.insert(name.clone(), StoredValue::from_value(value));
        }

        let mut functions = std::collections::HashMap::new();
        for (name, func) in self.env.iter_functions() {
            functions.insert(
                name.clone(),
                UserFunctionDef {
                    name: func.name.clone(),
                    params: func.params.clone(),
                    body: format!("{}", func.body), // Format the AST body as string
                },
            );
        }

        WorkspaceState {
            variables,
            functions,
        }
    }

    /// Restore workspace state from a snapshot
    fn restore_workspace(&mut self, workspace: &crate::storage::history::WorkspaceState) {
        use crate::core::env::UserFunction;
        use crate::core::parser;

        // Clear current workspace
        self.user_vars.clear();
        self.env.clear();

        // Restore variables
        for (name, value) in &workspace.variables {
            let val = value.to_value();
            self.env.set(name.clone(), val.clone());
            self.user_vars.insert(name.clone(), val);
        }

        // Restore functions
        for (_name, func_def) in &workspace.functions {
            // Try to parse the body string back to AST
            if let Ok(body_expr) = parser::parse(&func_def.body) {
                let func = UserFunction {
                    name: func_def.name.clone(),
                    params: func_def.params.clone(),
                    body: body_expr,
                };
                self.env.set_function(func);
            }
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{char_to_byte_index, App};
    use crate::tui::events::Action;

    #[test]
    fn char_to_byte_index_handles_multibyte_text() {
        assert_eq!(char_to_byte_index("α^3", 0), 0);
        assert_eq!(char_to_byte_index("α^3", 1), "α".len());
        assert_eq!(char_to_byte_index("α^3", 2), "α^".len());
        assert_eq!(char_to_byte_index("α^3", 99), "α^3".len());
    }

    #[test]
    fn partial_power_inputs_do_not_panic() {
        for input in ["^3", "^-"] {
            let mut app = App::new();
            for ch in input.chars() {
                app.handle_action(Action::InputChar(ch));
            }
            app.handle_action(Action::Eval);

            assert!(app.last_error.is_some(), "{input} should be reported as an error");
            assert!(app.running, "{input} should not stop the app");
        }
    }
}

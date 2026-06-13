pub fn get_system_prompt() -> String {
    include_str!("prompts/system_prompt.md").to_string()
}

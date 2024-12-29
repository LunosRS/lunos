pub fn colorize(message: &str, color: &str) {
    println!(
        "{}",
        match color {
            "red" => format!("\x1b[31m{}\x1b[0m", message),
            "yellow" => format!("\x1b[33m{}\x1b[0m", message),
            "green" => format!("\x1b[32m{}\x1b[0m", message),
            "blue" => format!("\x1b[34m{}\x1b[0m", message),
            "purple" | "pink" => format!("\x1b[35m{}\x1b[0m", message),
            "gray" => format!("\x1b[90m{}\x1b[0m", message),
            "white" => format!("\x1b[37m{}\x1b[0m", message),
            "black" => format!("\x1b[30m{}\x1b[0m", message),
            _ => message.to_string(),
        }
    );
}

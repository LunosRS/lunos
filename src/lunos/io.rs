pub fn colorize(message: &str, color: &str) {
    println!(
        "{}",
        match color {
            "red" => format!("\x1b[31m{message}\x1b[0m"),
            "yellow" => format!("\x1b[33m{message}\x1b[0m"),
            "green" => format!("\x1b[32m{message}\x1b[0m"),
            "blue" => format!("\x1b[34m{message}\x1b[0m"),
            "purple" | "pink" => format!("\x1b[35m{message}\x1b[0m"),
            "gray" => format!("\x1b[90m{message}\x1b[0m"),
            "white" => format!("\x1b[37m{message}\x1b[0m"),
            "black" => format!("\x1b[30m{message}\x1b[0m"),
            _ => message.to_string(),
        }
    );
}

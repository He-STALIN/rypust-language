const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

pub const SYNTAX_ERR: &str = "SyntaxError";
pub const INDENT_ERR: &str = "IndentationError";
pub const NAME_ERR: &str = "NameError";
pub const VALUE_ERR: &str = "ValueError";
pub const RUNTIME_ERR: &str = "RuntimeError";

#[macro_export]
macro_rules! raiseErr {
    ($error:expr, $($message:tt)*) => {
        // Мы форматируем сам $error, а всё остальное пропихиваем в форматирование сообщения
        eprintln!("\x1b[31m\x1b[1m{}\x1b[0m: {}", $error, format_args!($($message)*));
        std::process::exit(1);
    };
}

#[macro_export]
macro_rules! raiseWarn {
    ($warn:expr, $($message:tt)*) => {
        println!("\x1b[33m\x1b[1m{}\x1b[0m: {}", $warn, format_args!($($message)*));
    };
}

pub struct CliArgs {
    pub file_path: Option<String>,
    pub is_verbose: bool,
    pub is_quiet: bool,
    pub compile_string: Option<String>,
}

pub fn parse_args(args: &[String]) -> CliArgs {
    let mut config = CliArgs {
        file_path: None,
        is_verbose: false,
        is_quiet: false,
        compile_string: None,
    };

    if args.len() < 2 {
        /* запуск интерактивного режима */
    }

    let mut i = 1; // пропускаем имя программы
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                println!("Programming language '\x1b[1m{}\x1b[0m' by {} \nver. {}\n", NAME.to_uppercase(), AUTHORS, VERSION);
                println!("\x1b[32mUsage\x1b[0m: rypust [option] ... [-c/--compile '...' | file | -]
\x1b[32mAvaliable options\x1b[0m:
        \x1b[36m-h, --help      -> show this help message
        -c, --compile   -> compiling recieved code on next argument
        -V, --version   -> return language version and exit with code 0
        -v, --verbose   -> enable verbose
        -q, --quiet     -> don't write version of compiler and copyright on interactive startup. Ignoring with other options
        -C, --convert   -> convert python-style code to rypust-style
        --build         -> compile rypust code to exec byte-file
        -               -> start interactive compiler without agruments\x1b[0m");
                std::process::exit(0);
            }
            "-V" | "--version" => {
                println!("Compiler version: {}\nPackage Manager version: not realize", VERSION);
                std::process::exit(0);
            }
            "-v" | "--verbose" => config.is_verbose = true,
            "-q" | "--quiet" => config.is_quiet = true,
            "-c" | "--compile" => {
                if i + 1 < args.len() {
                    config.compile_string = Some(args[i+1].clone());
                    i += 1; // пропускаем следующий аргумент, так как это код
                } else {
                    raiseErr!("Exception", "-c require an argument");
                }
            },
            "--build" => {
                raiseWarn!("NotImplemented", "In next versions");
                std::process::exit(0);
            }
            _ => {
                // Если не флаг, значит это путь к файлу
                config.file_path = Some(args[i].clone());
            }
        }
        i += 1;
    }
    return config;
}
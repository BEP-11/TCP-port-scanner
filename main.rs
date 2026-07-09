use clap::Parser;
use colored::Colorize;
use std::net::SocketAddr;
use tokio::net::TcpStream;

// Структура для хранения аргументов командной строки
#[derive(Parser, Debug)]
#[command(name = "Rust Port Scanner")]
#[command(about = "Fast async TCP port scanner written in Rust")]
struct Args {
    /// IP адрес или доменное имя цели
    #[arg(short = 'H', long)]
host: String,

    /// Начальный порт (по умолчанию 1)
    #[arg(short, long, default_value_t = 1)]
    start_port: u16,

    /// Конечный порт (по умолчанию 1024)
    #[arg(short, long, default_value_t = 1024)]
    end_port: u16,
}

// Асинхронная функция для проверки одного порта
async fn scan_port(socket_addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    match TcpStream::connect(socket_addr).await {
        Ok(_) => {
            println!(
                "{} [{}] OPEN",
                "Port".green(),
                socket_addr.port().to_string().bold()
            );
            Ok(())
        }
        Err(e) => {
            // В реальном сканере мы обычно игнорируем ошибки закрытых портов,
            // чтобы не захламлять вывод, но можно добавить логирование.
            Err(e.into())
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Парсим аргументы командной строки
    let args = Args::parse();

    println!("Starting scan on {}...", args.host.cyan());
    println!("Range: {} - {}", args.start_port, args.end_port);

    // Создаем канал для ограничения количества одновременных соединений (Rate Limiting)
    // Это нужно, чтобы не сломать целевой сервер или не вызвать защиту.
    let num_workers = 100;
    let semaphore = tokio::sync::Semaphore::new(num_workers);

    for port in args.start_port..=args.end_port {
        let host = args.host.clone(); // Клонируем хост, так как он живет в замыкании
        let permit = semaphore.close(); // Клонируем семафор для управления потоками

        // Спавним новую асинхронную задачу (task)
        tokio::spawn(async move {
            // Ждем разрешения от семафора (ограничиваем количество concurrent запросов)
            let _permit = permit.acquire().await.unwrap();

            let addr = format!("{}:{}", host, port);
            let socket_addr: SocketAddr = match addr.parse() {
                Ok(addr) => addr,
                Err(_) => return, // Если парсинг IP не удался, прерываем эту задачу
            };

            if let Err(e) = scan_port(socket_addr).await {
                // Ошибка означает закрытый порт или таймаут, игнорируем для чистоты вывода
            }
        });
    }

    // Ждем завершения всех задач (необязательно для быстрых сканирований,
    // но хорошая практика для выхода программы)
    println!("Scan completed.");
    Ok(())
}

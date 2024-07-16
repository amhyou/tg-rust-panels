use serde_json::Value;
use std::error::Error;
use teloxide::{
    payloads::SendMessageSetters, prelude::*, types::ChatId, types::InputFile, types::Me,
    utils::command::BotCommands,
};

mod database;
mod keyboard;
mod user;
use user::*;

use redis::Commands;
use std::num::NonZeroUsize;
use warp::Filter;

use bytes::Bytes;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT};

// use image::Luma;
// use qrcode::QrCode;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

/// These are the supported commands:
#[derive(BotCommands)]
#[command(rename_rule = "lowercase")]
enum Command {
    /// Start
    Start,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    info!("Starting buttons bot...");

    let bot = Bot::from_env();

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(start))
        .branch(Update::filter_callback_query().endpoint(callback_handler));

    let add_cpanels = warp::post()
        .and(warp::path("add_cpanels"))
        .and(warp::body::content_length_limit(1024 * 32)) // Limit the size of the body
        .and(warp::body::bytes())
        .map(|body: Bytes| {
            let mut conn = database::REDIS_CLIENT.get_connection().unwrap();

            let body_str = String::from_utf8_lossy(&body);
            for line in body_str.lines() {
                let _: () = conn.lpush("cpanels", line).unwrap();
            }
            warp::reply::with_status("Lines added to queue", warp::http::StatusCode::CREATED)
        });

    let make_backup = warp::post().and(warp::path("make_backup")).map(|| {
        tokio::spawn(async {
            let bot2 = Bot::from_env();
            let _ = bot2
                .send_document(
                    ChatId(database::SUPPORT_ID),
                    InputFile::file("/data/dump.rdb"),
                )
                .await;
        });

        warp::reply::with_status("File sent", warp::http::StatusCode::OK)
    });

    let warp_server = tokio::spawn(async move {
        warp::serve(make_backup.or(add_cpanels))
            .run(([0, 0, 0, 0], 8000))
            .await;
    });

    // Spawn the Telegram bot
    let telegram_bot = tokio::spawn(async {
        Dispatcher::builder(bot, handler)
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;
    });

    // Wait for both tasks to complete
    let _ = tokio::join!(warp_server, telegram_bot);
    Ok(())
}

async fn start(bot: Bot, msg: Message, me: Me) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(text) = msg.text() {
        match BotCommands::parse(text, me.username()) {
            Ok(Command::Start) => {
                let userid = msg.chat.id.to_string();
                let user: User = User::load_from_redis(&userid).await;
                let balance = user.balance;
                bot.send_message(
                    msg.chat.id,
                    format!("Hello to Cpanels Seller\n\n🆔: {userid}\n💰 Balance: {balance} $"),
                )
                .reply_markup(keyboard::start_keyboard())
                .await?;
            }
            Err(_) => {
                bot.send_message(msg.chat.id, "Command not found!").await?;
            }
        }
    }
    Ok(())
}

async fn callback_handler(bot: Bot, q: CallbackQuery) -> Result<(), Box<dyn Error + Send + Sync>> {
    bot.answer_callback_query(q.id).await?;

    let message = q.message.unwrap();
    let userid = message.chat.id.to_string();
    let mut user: User = User::load_from_redis(&userid).await;

    match q.data {
        Some(callback) => {
            match callback.as_str() {
                "deposit" => {
                    // generate invoice link
                    let tron_address = user.get_invoice_address();

                    // let code = QrCode::new(&tron_address).unwrap();
                    // let image = code.render::<Luma<u8>>().build();
                    // image.save("./qrcode.png").unwrap();

                    // bot.send_photo(message.chat.id, InputFile::file("./qrcode.png"))
                    //     .await?;

                    bot.edit_message_text(
                        message.chat.id,
                        message.id,
                        format!("You can transfer USDT to this Tron wallet:\n\n {tron_address}\n"),
                    )
                    .reply_markup(keyboard::deposit_keyboard())
                    .await?;
                }
                "purchase" => {
                    bot.edit_message_text(
                        message.chat.id,
                        message.id,
                        format!("Choose the quantity you want to buy:"),
                    )
                    .reply_markup(keyboard::purchase_keyboard())
                    .await?;
                }
                "back" => {
                    let balance = user.balance;
                    bot.edit_message_text(
                        message.chat.id,
                        message.id,
                        format!("Hello to Cpanels Seller\n\n🆔: {userid}\n💰 Balance: {balance} $"),
                    )
                    .reply_markup(keyboard::start_keyboard())
                    .await?;
                }
                "paid" => {
                    let tron_address = user.get_invoice_address();

                    let url = format!(
                        "https://api.trongrid.io/v1/accounts/{tron_address}/transactions/trc20?only_confirmed=true&contract_address=TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t&only_to=true"
                    );

                    let mut headers = HeaderMap::new();
                    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
                    headers.insert(
                        "TRON-PRO-API-KEY",
                        HeaderValue::from_static(database::TRON_GRID),
                    );

                    let client = reqwest::Client::new();
                    let response = client.get(url).headers(headers).send().await?;

                    let body: Value = response.json().await?;

                    if let Some(data) = body["data"].as_array() {
                        if data.is_empty() {
                            // send a toasst to tell user that no payment is captured yet
                            bot.send_message(
                                message.chat.id,
                                format!("Payment is not yet confirmed, please wait !"),
                            )
                            .await?;
                            return Ok(());
                        } else {
                            let value =
                                data.get(0).unwrap().get("value").unwrap().as_str().unwrap();
                            let usdt_value = value.parse::<f64>().unwrap() / 1_000_000.0;
                            let new_balance = user.balance + usdt_value;
                            let new_invoice = user.invoice + 1;

                            user.balance = new_balance;
                            user.invoice = new_invoice;
                            user.save_to_redis().await;

                            bot.edit_message_text(
                                message.chat.id,
                                message.id,
                                format!("Hello to Cpanels Seller\n\n🆔: {userid}\n💰 Balance: {new_balance} $"),
                            )
                            .reply_markup(keyboard::start_keyboard())
                            .await?;

                            bot.send_message(
                                message.chat.id,
                                format!("+ {usdt_value} $ added to your balance."),
                            )
                            .await?;
                            // payment is made
                            bot.send_message(
                                ChatId(database::SUPPORT_ID),
                                format!(
                                    "{usdt_value} $ is added to {userid} in invoice {new_invoice}"
                                ),
                            )
                            .await?;
                        }
                    } else {
                        println!("The data field is not an array.");
                    }
                }
                s if s.starts_with("buy_") => {
                    // check the balance if not zero
                    if user.balance < 1.0 {
                        bot.send_message(
                            message.chat.id,
                            format!("You have insufficient balance !"),
                        )
                        .await?;
                        return Ok(());
                    }

                    // buy some items
                    let parts: Vec<&str> = callback.split('_').collect();
                    let quantity = parts[1].parse::<usize>()?;

                    let mut conn = database::REDIS_CLIENT.get_connection().unwrap();
                    let non_zero_quantity = NonZeroUsize::new(quantity).unwrap();
                    let results: Vec<String> =
                        conn.rpop("cpanels", Some(non_zero_quantity)).unwrap();

                    let qty_inventory = results.len() as u16;

                    let deductible = std::cmp::min(user.balance as u16, qty_inventory);

                    if deductible < 1 {
                        bot.send_message(message.chat.id, format!("No sufficient items to buy"))
                            .await?;
                    } else {
                        user.balance -= deductible as f64;
                        user.save_to_redis().await;

                        bot.send_message(
                            message.chat.id,
                            format!("You bought {} items:\n{}", deductible, results.join("\n")),
                        )
                        .await?;
                    }
                }
                "support" => {
                    bot.edit_message_text(
                        message.chat.id,
                        message.id,
                        format!(
                            "If you have any problem you can ask the support @RustaceanSupport"
                        ),
                    )
                    .reply_markup(keyboard::support_keyboard())
                    .await?;
                }
                _ => error!("no existing case"),
            }
        }
        None => {
            error!("there is some callback error");
        }
    };
    Ok(())
}

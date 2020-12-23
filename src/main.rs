use chrono::{prelude::*, Duration};
use clap::{crate_authors, App, Arg};
use tokio::fs::File;
use tokio::prelude::*;

async fn get_exchange_rate(date: &str) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::builder().use_rustls_tls().build().unwrap();

    let body = client
        .get(&format!(
            "https://www.nbrb.by/Services/XmlExRates.aspx?ondate={}",
            date
        ))
        .send()
        .await?
        .text()
        .await?;

    Ok(body)
}

async fn write_to_xml_file(data: &[u8], file_name: &str) -> Result<(), std::io::Error> {
    let mut file = File::create(format!("{}.xml", file_name)).await?;
    file.write_all(data).await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    let matches = App::new("Загрузчик курсов валют с сайта НБРБ")
        .version("0.1")
        .author(crate_authors!())
        .about("Скачивает xml файлы с курсами валют с сайта NBRB за указанный период.")
        .arg(
            Arg::with_name("from_date").value_name("DATE")
                .short("f")
                .long("from")
                .required(true)
                .help("Указывает дату, начиная с которой необходимо получить курсы валют. Формат - 'MM/DD/YYYY'."),
        )
        .arg(
            Arg::with_name("to_date").value_name("DATE")
                .short("t")
                .long("to")
                .required(true)
                .help("Указывает дату, заканчивая которой необходимо получить курсы валют. Формат - 'MM/DD/YYYY'."),
        ).get_matches();

    let from_date_string = matches.value_of("from_date").unwrap();
    let to_date_string = matches.value_of("to_date").unwrap();

    if from_date_string == to_date_string {
        let exchange_rate = get_exchange_rate(from_date_string).await.unwrap();
        write_to_xml_file(
            exchange_rate.as_bytes(),
            &from_date_string.replace('/', "."),
        )
        .await
        .unwrap();

        return;
    }

    let from_date = NaiveDate::parse_from_str(from_date_string, "%m/%d/%Y").unwrap();
    let to_date = NaiveDate::parse_from_str(to_date_string, "%m/%d/%Y").unwrap();

    let mut date = from_date.clone();

    let mut handles = vec![];
    while date <= to_date {
        let date_clone = date.clone();
        handles.push(tokio::spawn(async move {
            let exchange_rate = get_exchange_rate(&date_clone.format("%m/%d/%Y").to_string())
                .await
                .unwrap();
            write_to_xml_file(
                exchange_rate.as_bytes(),
                &date_clone.format("%m.%d.%Y").to_string(),
            )
            .await
            .unwrap();
        }));

        date += Duration::days(1);
    }
    futures::future::join_all(handles).await;
}

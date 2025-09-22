use std::{collections::HashMap, fs, path::PathBuf};

use rayon::prelude::*;
use regex::Regex;

use crate::{
    dc_structs::{DiscordMessage, DiscordUser},
    errors::ErrorThingy,
};

pub fn analyze(package_path: PathBuf) -> Result<(), ErrorThingy> {
    let account = fs::read_to_string(package_path.join("Account/user.json"))
        .expect("Something went wrong reading the file");
    let account: DiscordUser = serde_json::from_str(&account).unwrap();

    println!("[+] analyzing data dump for {}", account.username);

    let mut money_spent: i32 = 0;
    for payment in account.money_wastes {
        let money_change = payment.amount - payment.amount_refunded;

        money_spent += money_change as i32;
    }

    println!(" money wasted on discord nitro: {} USD", money_spent / 100);

    println!("[+] analyzing messages idfk");

    get_message_counts(package_path.join("Messages"))?;

    Ok(())
}

fn get_message_counts(data_path: PathBuf) -> Result<(), ErrorThingy> {
    use std::sync::Mutex;
    let total_message_count = Mutex::new(0u32);
    let total_messages_with_attachements = Mutex::new(0u32);
    let direct_messages = Mutex::new(0u32);
    let word_counts = Mutex::new(HashMap::new());

    // i HATE regex!!! but i still need it!!!
    let url_re = Regex::new(r"https?://(www\.)?[-a-zA-Z\d@:%._+~#=]{1,256}\.[a-zA-Z\d()]{1,6}\b([-a-zA-Z\d()!@:%_+.~#?&/=]*)").unwrap();

    // Manually map the error to ErrorThingy
    let entries: Vec<_> = match data_path.read_dir() {
        Ok(rd) => rd.filter_map(Result::ok).collect(),
        Err(e) => return Err(ErrorThingy::Io(e)), // Adjust this variant to match your enum
    };

    entries.par_iter().for_each(|entry| {
        let path = entry.path();
        if path.is_dir() {
            let message_channel = fs::read_to_string(path.join("channel.json"))
                .expect("Something went wrong reading the file");
            let _message_channel_parsed =
                json::parse(&message_channel).expect("Failed to parse JSON");

            let messages_in_channel: u32 = 0;
            let messages_attachments_in_channel: u32 = 0;

            // i wish i remembered what this does
            /*if message_channel_parsed["type"].as_i8().unwrap() == 1 {
                return;
            }*/

            let messages: Vec<DiscordMessage> =
                serde_json::from_reader(fs::File::open(path.join("messages.json")).unwrap())
                    .unwrap();

            let mut local_word_counts = HashMap::new();

            for message in messages {
                // i reeeaaalllyyy wanna get rid of this regex but i do NOT want to implement url matching :(
                let content = url_re.replace_all(&message.content, "");

                // this sucks for performance, but i can't use retain because i need to replace w/ space
                let content: String = content
                    .chars()
                    .map(|c| if !c.is_ascii_alphanumeric() { ' ' } else { c })
                    .collect();

                content.to_lowercase().split_whitespace().for_each(|word| {
                    if word.len() > 1 && !word.chars().all(|c| c.is_ascii_digit()) {
                        *local_word_counts.entry(word.to_string()).or_insert(0) += 1;
                    }
                });
            }

            let mut global_word_counts = word_counts.lock().unwrap();
            for (word, count) in local_word_counts {
                *global_word_counts.entry(word).or_insert(0) += count;
            }

            *total_message_count.lock().unwrap() += messages_in_channel;
            *total_messages_with_attachements.lock().unwrap() += messages_attachments_in_channel;

            /*if message_channel_parsed["type"].as_i8().unwrap() == 1 {
                direct_messages += messages_in_channel;
            }*/
        }
    });

    println!("Total messages: {}", *total_message_count.lock().unwrap());
    println!(
        "Messages with attachments: {}",
        *total_messages_with_attachements.lock().unwrap()
    );
    println!("Direct messages: {}", *direct_messages.lock().unwrap());

    println!("[+] analyzing word counts ...");
    println!(
        "Total different words: {}",
        word_counts.lock().unwrap().len()
    );

    let binding = word_counts.lock().unwrap();
    let mut count_vec: Vec<_> = binding.iter().collect();
    count_vec.sort_by(|a, b| b.1.cmp(a.1));

    println!("Most frequent words: ");
    let mut i = 0;
    for (word, count) in count_vec.iter().take(20) {
        i += 1;
        println!("{}: {} ({})", i, word, count);
    }

    Ok(())
}

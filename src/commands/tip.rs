use serenity::utils::MessageBuilder;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::path::Path;
use chrono::{Local, TimeZone};

use std::time::{SystemTime, UNIX_EPOCH};

const SECONDS_IN_WEEK: u64 = 604800;
const WEEKLY_TIPS: u8 = 7;
const WEEKLY_ANTI_TIPS: u8 = 1;

#[derive(Serialize, Deserialize, Clone)]
struct User {
    user_id: u64,
    lifetime_tips: i32,
    week_tips: i8,
    tips_to_give: u8,
    tips_given: u32,
    #[serde(default = "default_anti_tip")]
    anti_tips: u8,
    #[serde(default)]
    anti_tips_given: u32,
}

fn default_anti_tip() -> u8 {
    WEEKLY_ANTI_TIPS
}

#[derive(Serialize, Deserialize)]
struct Data {
    reset_time: u64,
    users: Vec<User>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Tip {
    tipper_id: u64,
    tipper_name: String,
    tipee_id: u64,
    tipee_name: String,
    time: u64,
    #[serde(default = "default_is_anti")]
    is_anti: bool,
}

fn default_is_anti() -> bool {
    false
}

#[derive(Serialize, Deserialize)]
struct Tips {
    tips: Vec<Tip>,
}

command!(tip_log(_ctx, msg) {
    let log_path = Path::new("./data/tips/log.json");
    let json_log = match OpenOptions::new().read(true).open(log_path) {
        Ok(f) => f,
        Err(_) => OpenOptions::new().write(true).create(true).open(log_path).expect("Error creating log file"),
    };

    let mut tip_data: Tips = match serde_json::from_reader(json_log) {
        Ok(j) => j,
        Err(_) => Tips {
            tips: Vec::new()
        }
    }; 

    let mut out: Vec<Tip> = Vec::new();

    let len = tip_data.tips.len();
    if len > 5 {
        for i in 1..=5 {
            out.push(tip_data.tips[len - i].clone()); 
        }
    } else {
        out = tip_data.tips;
        out.reverse();
    }

    let mut content = MessageBuilder::new()
            .push("```md\n")
            .push("### Most Recent Tips ###");

    for entry in out {
        let tip_text = if entry.is_anti {
            "anti tipped"
        } else {
            "tipped"
        };
        content = content.push(format!("\n* [{}] {} {} {}", Local.timestamp(entry.time as i64,0), 
                             entry.tipper_name, 
                             tip_text,
                             entry.tipee_name));
    }

    let resp = content.push("\n```").build();
    let _ = msg.channel_id.say(&resp);
});

fn user_from_id(id: u64) -> User {
    let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let path = Path::new("./data/tips/tips.json");
    let json = match OpenOptions::new().write(true).read(true).open(path) {
        Ok(f) => f,
        Err(_) => OpenOptions::new().write(true).create(true).open(path).expect("Error creating file"),
    };

    let mut data: Data = match serde_json::from_reader(json) {
        Ok(j) => j,
        Err(_) => Data {
            reset_time: time.as_secs() + SECONDS_IN_WEEK,
            users: Vec::new(),
        }
    };

    let exists = data.users.iter().any(|x| x.user_id == id);

    if exists {
        data.users.iter_mut().find(|x| x.user_id == id).unwrap().clone()
    } else {
        data.users.push(User {
            user_id: id,
            lifetime_tips: 0,
            week_tips: 0,
            tips_to_give: WEEKLY_TIPS,
            tips_given: 0,
            anti_tips: WEEKLY_ANTI_TIPS,
            anti_tips_given: 0,
        });
        let len = data.users.len();                
        let writer = BufWriter::new(OpenOptions::new().write(true).open(path).unwrap());
        let _  = serde_json::to_writer(writer, &data).unwrap();
        data.users.get_mut(len - 1).unwrap().clone()
    }
}

command!(profile(_ctx, msg, msg_args) {
    let is_other = match msg_args.single_n::<serenity::model::id::UserId>() {
        Ok(_) => true,
        Err(_) => false,
    };

    let user: serenity::model::user::User = if is_other {
        msg_args.single::<serenity::model::id::UserId>().unwrap().to_user().unwrap()
    } else {
        msg.author.clone()
    };

    let tip_user: User = user_from_id(*user.id.as_u64());
    let avatar = user.face();
    let ava_url = if avatar.ends_with(".webp?size=1024") {
        &avatar[..avatar.len() - 15]
    } else {
        avatar.as_str()
    };

    let _ = msg.channel_id.send_message(|m| m
                                        .embed(|e| e
                                               .title("Tip Profile")
                                               .thumbnail(
                                                   ava_url
                                                   )
                                               .field("Tips Recieved: All Time",
                                                      tip_user.lifetime_tips,
                                                      true)
                                               .field("Tips Recieved: This Week",
                                                      tip_user.week_tips,
                                                      true)
                                               .field("Tips Given: All Time",
                                                      tip_user.tips_given,
                                                      false)
                                               .field("Weekly Tips Remaining",
                                                      tip_user.tips_to_give,
                                                      true)
                                               .field("Anti Tips Given: All Time",
                                                      tip_user.anti_tips_given,
                                                      true)
                                               .field("Weekly Anti Tips Remaining",
                                                      tip_user.anti_tips,
                                                      true)
                                               .color(
                                                    serenity::utils::Colour::GOLD
                                                )
                                               ));

});

command!(tip(_ctx, msg, msg_args) {
    let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let path = Path::new("./data/tips/tips.json");
    let json = match OpenOptions::new().write(true).read(true).open(path) {
        Ok(f) => f,
        Err(_) => OpenOptions::new().write(true).create(true).open(path).expect("Error creating file"),
    };

    let is_anti = msg.content.starts_with("-anti");

    let mut data: Data = match serde_json::from_reader(json) {
        Ok(j) => j,
        Err(_) => Data {
            reset_time: time.as_secs() + SECONDS_IN_WEEK,
            users: Vec::new(),
        }
    };

    let log_path = Path::new("./data/tips/log.json");
    let json_log = match OpenOptions::new().write(true).read(true).open(log_path) {
        Ok(f) => f,
        Err(_) => OpenOptions::new().write(true).create(true).open(log_path).expect("Error creating log file"),
    };

    let mut tip_data: Tips = match serde_json::from_reader(json_log) {
        Ok(j) => j,
        Err(_) => Tips {
            tips: Vec::new()
        }
    };


    if time.as_secs() > data.reset_time {
        data.users.iter_mut().for_each(|x| {x.tips_to_give = WEEKLY_TIPS; x.week_tips = 0;});
        while data.reset_time < time.as_secs() {
            data.reset_time += SECONDS_IN_WEEK;
        }
    }

    let tipper_id = msg.author.id;
    let tipper = *tipper_id.as_u64();
    let mut tipee_id: serenity::model::id::UserId;
    let is_tip = match msg_args.single_n::<serenity::model::id::UserId>() {
        Ok(_) => true,
        Err(_) => false,
    };
    if !is_tip {
        let exists = data.users.iter().any(|x| x.user_id == tipper);
        let mut tipper_user = if exists {
            data.users.iter_mut().find(|x| x.user_id == tipper).unwrap()
        } else {
            data.users.push(User {
                user_id: tipper,
                lifetime_tips: 0,
                week_tips: 0,
                tips_to_give: WEEKLY_TIPS,
                tips_given: 0,
                anti_tips: WEEKLY_ANTI_TIPS,
                anti_tips_given: 0,
            });
            let len = data.users.len();
            data.users.get_mut(len - 1).unwrap()
        };

        let mut content = MessageBuilder::new()
            .push("```md\n")
            .push(format!("\n# You have:\n* Lifetime recieved tips: {}", tipper_user.lifetime_tips))
            .push(format!("\n* Tips recieved this week: {}", tipper_user.week_tips))
            .push(format!("\n* Lifetime tips given: {}", tipper_user.tips_given))
            .push(format!("\n* Tips to give this week: {}", tipper_user.tips_to_give))
            .push(format!("\n* Lifetime anti tips to given: {}", tipper_user.anti_tips_given))
            .push(format!("\n* Anti tips to give this week: {}", tipper_user.anti_tips))
            .push(format!("\n\n### Usage ###\n -tip @some_well_deserving_person_here\n"))
            .push(format!("\n### Info ###\nNext weekly tips reset: {} \n", Local.timestamp(
                                                                                    data.reset_time as i64,
                                                                                    0)))
            .push(format!("\n```"))
            .build();
        let _ = msg.reply(&content);
    } else {
        tipee_id = match msg_args.single::<serenity::model::id::UserId>() {
            Ok(u) => u,
            Err(_) => panic!(),
        };

        let tipee = *tipee_id.as_u64();

        if tipper == tipee {
            let _ = msg.reply("You can't tip yourself lol");
        } else {
            let mut _tipper_to_give = 0;
            let mut _tipper_given = 0;
            let mut _tipee_tips = (0, 0);
            let mut _tipee_name: String;
            let mut has_tips = true;
            {
                let exists = data.users.iter().any(|x| x.user_id == tipper);
                let mut tipper_user = if exists {
                    data.users.iter_mut().find(|x| x.user_id == tipper).unwrap()
                } else {
                    data.users.push(User {
                        user_id: tipper,
                        lifetime_tips: 0,
                        week_tips: 0,
                        tips_to_give: WEEKLY_TIPS,
                        tips_given: 0,
                        anti_tips: WEEKLY_ANTI_TIPS,
                        anti_tips_given: 0,
                    });
                    let len = data.users.len();
                    data.users.get_mut(len - 1).unwrap()
                };



                if !is_anti {
                    if tipper_user.tips_to_give == 0 {
                        let _ = msg.reply("You are out of tips this week, try again next week");
                        has_tips = false;
                    } else {
                        tipper_user.tips_to_give -= 1;
                        tipper_user.tips_given += 1;
                        _tipper_to_give = tipper_user.tips_to_give;
                        _tipper_given = tipper_user.tips_given;
                    }
                } else {
                    if tipper_user.anti_tips == 0 {
                        let _ = msg.reply("You are out of anti tips this week, try again next week");
                        has_tips = false;
                    } else {
                        tipper_user.anti_tips -= 1;
                        tipper_user.anti_tips_given += 1;
                        _tipper_to_give = tipper_user.anti_tips;
                        _tipper_given = tipper_user.anti_tips_given;
                    }
                }
            }
            if has_tips {
                {

                    let exists = data.users.iter().any(|x| x.user_id == tipee);
                    let mut tipee_user = if exists {
                        data.users.iter_mut().find(|x| x.user_id == tipee).unwrap()
                    } else {
                        data.users.push(User {
                            user_id: tipee,
                            lifetime_tips: 0,
                            week_tips: 0,
                            tips_to_give: WEEKLY_TIPS,
                            tips_given: 0,
                            anti_tips: WEEKLY_ANTI_TIPS,
                            anti_tips_given: 0,
                        });
                        let len = data.users.len();
                        data.users.get_mut(len - 1).unwrap()
                    };

                    if !is_anti {
                        tipee_user.week_tips += 1;
                        tipee_user.lifetime_tips += 1;
                    } else {
                        tipee_user.week_tips -= 1;
                        tipee_user.lifetime_tips -= 1;
                    }
                    _tipee_tips = (tipee_user.week_tips, tipee_user.lifetime_tips);
                    _tipee_name = tipee_id.to_user().unwrap().name;
                }

                let tip_text = if is_anti {
                    " anti tipped"
                } else {
                    " tipped "
                };
                let giver_text = if is_anti {
                    format!("has given {} lifetime anti tips, and has {} remaining anti tips this week\n", 
                            _tipper_given, _tipper_to_give)
                } else {
                    format!(" has given {} lifetime tips and has {} remaining this week \n",
                            _tipper_given,
                            _tipper_to_give)
                };
                let mut content = MessageBuilder::new()
                    .mention(&tipper_id)
                    .push(tip_text)
                    .mention(&tipee_id)
                    .push("\n")
                    .mention(&tipper_id)
                    .push(giver_text)
                    .mention(&tipee_id)
                    .push(format!(" has recieved {} tips this week and {} lifetime tips", _tipee_tips.0, _tipee_tips.1))
                    .build();

                println!("{:?} -> {:?}" , tipper, tipee);
                let _ = msg.channel_id.say(&content);
                let writer = BufWriter::new(OpenOptions::new().write(true).open(path).unwrap());
                let _  = serde_json::to_writer(writer, &data).unwrap();

                let transaction = Tip {
                    tipper_id: tipper,
                    tipper_name: msg.author.name.clone(),
                    tipee_id: tipee,
                    tipee_name: _tipee_name,
                    time: time.as_secs(),
                    is_anti,
                };

                tip_data.tips.push(transaction);
            }
            let log_writer = BufWriter::new(OpenOptions::new().write(true).open(log_path).unwrap());
            let _ = serde_json::to_writer(log_writer, &tip_data);
        }
    }
});
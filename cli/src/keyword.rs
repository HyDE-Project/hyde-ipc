pub fn sync_keyword(get: bool, set: bool, keyword: String, value: Option<String>) {
    if get {
        match hyprland::keyword::Keyword::get(&keyword) {
            Ok(result) => println!("{} value is {}", keyword, result.value),
            Err(e) => eprintln!("Error getting keyword '{}': {}", keyword, e),
        }
    } else if set {
        match value {
            Some(ref val) => {
                if let Err(e) = hyprland::keyword::Keyword::set(keyword.clone(), val.clone()) {
                    eprintln!("Error setting keyword '{}': {}", keyword, e);
                }
            },
            None => eprintln!("Error: value required for set operation"),
        }
    }
}

pub async fn async_keyword(get: bool, set: bool, keyword: String, value: Option<String>) {
    if get {
        match hyprland::keyword::Keyword::get_async(&keyword).await {
            Ok(result) => println!("{} value is {}", keyword, result.value),
            Err(e) => eprintln!("Error getting keyword '{}': {}", keyword, e),
        }
    } else if set {
        match value {
            Some(ref val) => {
                if let Err(e) =
                    hyprland::keyword::Keyword::set_async(keyword.clone(), val.clone()).await
                {
                    eprintln!("Error setting keyword '{}': {}", keyword, e);
                }
            },
            None => eprintln!("Error: value required for set operation"),
        }
    }
}

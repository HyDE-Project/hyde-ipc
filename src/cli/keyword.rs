pub fn sync_keyword(get: bool, set: bool, keyword: String, value: Option<String>) {
    let x = 1;
    if get {
        println!(
            "{} value is {}",
            keyword,
            hyprland::keyword::Keyword::get(&keyword)
                .unwrap()
                .value
        );
    } else if set {
        let value = value.as_ref().unwrap();
        hyprland::keyword::Keyword::set(keyword, value.clone()).unwrap();
    }
}

pub async fn async_keyword(get: bool, set: bool, keyword: String, value: Option<String>) {
    if get {
        println!(
            "{} value is {}",
            keyword,
            hyprland::keyword::Keyword::get_async(&keyword)
                .await
                .unwrap()
                .value
        );
    } else if set {
        let value = value.as_ref().unwrap();
        hyprland::keyword::Keyword::set_async(keyword, value.clone())
            .await
            .unwrap();
    }
}

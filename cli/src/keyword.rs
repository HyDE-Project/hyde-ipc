/// Reads or writes one keyword and hands every failure back to the caller.
///
/// Reporting the error here instead of returning it would leave the CLI exiting
/// with a success status on a keyword that was never read or written.
pub fn sync_keyword(
    get: bool,
    set: bool,
    keyword: String,
    value: Option<String>,
) -> Result<(), String> {
    if get {
        let result = hyprland::keyword::Keyword::get(&keyword)
            .map_err(|e| format!("getting keyword '{keyword}': {e}"))?;
        println!("{} value is {}", keyword, result.value);
    } else if set {
        let value = value.ok_or_else(|| "value required for set operation".to_string())?;
        hyprland::keyword::Keyword::set(keyword.clone(), value)
            .map_err(|e| format!("setting keyword '{keyword}': {e}"))?;
    }

    Ok(())
}

/// Asynchronous counterpart of [`sync_keyword`], with the same error contract.
pub async fn async_keyword(
    get: bool,
    set: bool,
    keyword: String,
    value: Option<String>,
) -> Result<(), String> {
    if get {
        let result = hyprland::keyword::Keyword::get_async(&keyword)
            .await
            .map_err(|e| format!("getting keyword '{keyword}': {e}"))?;
        println!("{} value is {}", keyword, result.value);
    } else if set {
        let value = value.ok_or_else(|| "value required for set operation".to_string())?;
        hyprland::keyword::Keyword::set_async(keyword.clone(), value)
            .await
            .map_err(|e| format!("setting keyword '{keyword}': {e}"))?;
    }

    Ok(())
}

use elevenlabs_rs::{Result, Speech};

pub async fn speak(say: String, character: String) -> Result<()> {
    let speech = Speech::new(&say, &character, "eleven_turbo_v2", 0).await?;

    speech.play()?;

    // None will generate a filename with the voice name and the current utc timestamp
    // e.g. Clyde_1624299999.mp3
    speech.save(None)?; // or speech.save(Some("my_file_name.mp3".to_string()))?;

    Ok(())
}

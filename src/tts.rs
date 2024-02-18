// import the dependencies
use elevenlabs_api::{
    tts::{TtsApi, TtsBody},
    *,
};
use uuid::Uuid;
pub fn say(message: String, character: String) -> String {
    // Load API key from environment ELEVENLABS_API_KEY.
    // You can also hadcode through `Auth::new(<your_api_key>)`, but it is not recommended.
    let auth = Auth::from_env().unwrap();
    let elevenlabs = Elevenlabs::new(auth, "https://api.elevenlabs.io/v1/");

    let voice_id = match character.as_str() {
        "Clyde" => "2EiwWnXFnvU5JabPnv8n",
        "Glinda" => "z9fAnlkpzviPz146aGWa",
        _ => "SOYHLrjzK2X1ezoPC6cr",
    };

    // Create the tts body.
    let tts_body = TtsBody {
        model_id: Some(String::from("eleven_turbo_v2")),
        text: message,
        voice_settings: None,
    };

    // Generate the speech for the text by using the voice with id yoZ06aMxZJJ28mfd3POQ.
    let tts_result = elevenlabs.tts(&tts_body, voice_id);
    let bytes = tts_result.unwrap();

    // Do what you need with the bytes.
    // The server responds with "audio/mpeg" so we can save as mp3.
    let file_name = format!("{}.mp3", Uuid::new_v4());
    std::fs::write(format!("assets/{}", file_name.clone()), bytes).unwrap();
    file_name
}

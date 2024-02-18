<div align="center">
  <img src="https://github.com/BALD-rs/lucky-liars/blob/main/logo.png?raw=true" width="700 height="400">
<h1>Lucky Liars</h1>
  <h3>Powered by Rust 🦀</h3>
</div>

## Lucky Liars is AI-powered murder mystery game that generates a new experience every playthrough.
<br>
<br>
Playing as Detective Derek Dynamo of the FBI, you are investigating a murder case. The twist: the murder case is different every time you play the game! You must interview the same 3 suspects every time, and the true killer is always among them, though it will be a different suspect in different playthroughs. You are armed with a powerful tool: the Probability Polygraph, which rolls a die from 1 to 20 each time it observes a potential lie. The closer you are to 20, the more likely it is that spikes on the polygraph represent an actual lie.
<br>
<br>

# Technology used:


<img align="left" width="170" src="https://bevyengine.org/assets/bevy_logo_dark.svg" />


## Bevy Engine 
<h3>The Bevy game engine is an open source engine written in Rust, and it powered our game client. We used it for its speed and portability on any system. </h3>

<br>

<img align="left" width="170" src="https://github.com/BALD-rs/lucky-liars/assets/65707789/804812c0-080a-4025-a649-6848cfd04fb3" />

## Blender

<h3>Blender was used to create all the assets for the game during the hackathon. All 3d models were made locally, with their textures taken from images inside the Johnny Carson Center for Emerging Media Arts</h3>
<br>

<img align="left" width="170" src="https://media.licdn.com/dms/image/D4E12AQEBg943ptCYpg/article-cover_image-shrink_720_1280/0/1686391647921?e=2147483647&v=beta&t=sTfwUvcIfW7Fuby7hMluDfuRJK3HfYMMWc2SyZR7-GA" />

## Express.js

<h3>The API was written in Node.js using the Express web framework for handling the AI characters and story generation. A separate API server is needed in order to handle persistence of the characters' backstories and personalities throughout the interrogation, as well as handling generation of a new murder mystery and a new killer every playthrough. The code for the API can be found in <a href="https://github.com/BALD-rs/cornhacks24-api">this repository</a>.</h3>
<br>

<img align="left" width="170" src="https://github.com/BALD-rs/cornhacks24-game/assets/65707789/2eceec9c-cbd0-41a2-985d-3da28bad3e61" />

## Hardware

<h3>The game is made more realistic through the use of an intercom button and nixie tubes to display the numbers. The peripheral is required to enable speech-to-text, making the game more realistic, as if the user was pressing an intercom button. After the user releases the button, a D20 is rolled and the result is displayed using the nixie tubes with a special animation. This value is similar to a "perception check," and the result is crucial to determining the true killer. The code for the hardware can be found in <a href="https://github.com/BALD-rs/lucky-liars-hardware">this repository</a>.</h3>

## Developed by
- Blaine Traudt
- Anton Angeletti
- Louis Quattrocchi
- Dawson McGahan


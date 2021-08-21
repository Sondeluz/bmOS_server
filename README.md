# bmOS_server

bmOS_server is an executable in charge of receiving intents and rendering their associated BMO-faces and 
playing audio tracks. This is part of the software required to run my custom BMO-Boy. Images and a blog post are coming soon. More in-depth documentation is available [here](https://github.com/Sondeluz/bmOS_server).

The documentation and setup information for bmOS_client, the other software component which sends the intents to this one, is also available [here](https://docs.rs/bmos_client/), with the source located [here](https://github.com/Sondeluz/bmOS_client)



## Configuration files
The following configuration files are required to be present in the same folder the executable is in:
- **faces.txt** : Indicates the image files of BMO's faces to be shown for each intent. It's mandatory to have at least one entry for each intent which will be sent from the client, except for the weather and chronometer functionalities preset intents. Otherwise, the application will panic.
- **audio.txt** : Indicates the audio tracks to be played for each intent. It can be empty
- **timings.txt** : Indicates the time limits for each intent. It is mandatory to have one entry for each intent without an audio track, again excluding the preset intents.

**Information about the syntax and contents needed in each of the configuration files is present in the documentation of the functions inside the config module.**

## Mandatory intents
The following intents are mandatory to have faces defined in faces.txt:
- **"default"**: In order to show BMO's default/fallback face.

The following files are mandatory to be present in the executables folder:
- **./assets/faces/alarm.jpg** : Alarm face to be shown after a chronometer finishes.
- **./assets/audio/alarm.wav** : Alarm audio track to be played after a chronometer finishes.
- **./assets/font.ttf** : Font to be used when showing text. I recommend [Video Terminal Screen](https://ttfonts.net/en/download/62485.htm)

## Shutdown
- bmOS_server listens for key inputs. If the escape key is pressed, the application will exit.
- If it's running in a headless server, the appropiate way of exiting is to close bmOS_client (or any other source) which is sending intents to it. This will trigger a safe shutdown.
## Assumptions
The following assumptions are made when running this application:
- openAL, SDL2 and SDL2-ttf libraries are installed in the system
- If the device running the bmOS_server is the audio (microphone) source, it's streaming it to the device running bmOS_client by some other means. bmOS_server does not record any audio, and only listens to strings received to its provided address.

## Recommendations
- Since the paths for the mandatory files are pre-determined, I advise to have all assets in an "assets" folder wherever the executable is in.
- Provide appropiate timings for each intent. For example, setting 100 for a given intent without an audio track will make it zoom past it and go back to the default state. A good timing that I found for such intents is 4500 (4'5 seconds).
- I used all kinds of image and audio formats with success and stuck with .jpg and .png for images, and .ogg and .wav for audio. The more compressed the better since they are read from disk, and devices such as a raspberry are really slow to read them.

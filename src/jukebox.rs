use serde_json;
use serde::de::{Deserialize, Deserializer};
use std::result;

use client::Client;
use error::Result;
use media::song::Song;
use query::Query;

#[derive(Debug)]
pub struct Jukebox<'a> {
    client: &'a mut Client,
}

#[derive(Debug, Deserialize)]
pub struct JukeboxStatus {
    #[serde(rename = "currentIndex")]
    pub index: isize,
    pub playing: bool,
    #[serde(rename = "gain")]
    pub volume: f32,
    pub position: usize,
}

#[derive(Debug)]
pub struct JukeboxPlaylist {
    pub status: JukeboxStatus,
    pub songs: Vec<Song>,
}

impl<'de> Deserialize<'de> for JukeboxPlaylist {
    fn deserialize<D>(de: D) -> result::Result<Self, D::Error> where
        D: Deserializer<'de>
    {
        #[derive(Deserialize)]
        struct _Playlist {
            #[serde(rename = "currentIndex")]
            index: isize,
            playing: bool,
            gain: f32,
            position: usize,
            entry: Vec<Song>,
        }
        let raw = _Playlist::deserialize(de)?;
        Ok(JukeboxPlaylist {
            status: JukeboxStatus {
                index: raw.index,
                playing: raw.playing,
                volume: raw.gain,
                position: raw.position,
            },
            songs: raw.entry,
        })
    }
}

impl<'a> Jukebox<'a> {
    /// Creates a new handler to the jukebox of the client.
    pub fn start(client: &'a mut Client) -> Jukebox {
        Jukebox { client }
    }

    fn send_action_with<U>(
        &mut self,
        action: &str,
        index: U,
        ids: Vec<usize>,
    ) -> Result<JukeboxStatus>
    where
        U: Into<Option<usize>>,
    {
        let args = Query::with("action", action)
            .arg("index", index.into())
            .arg_list("id", ids)
            .build();
        let res = self.client.get("jukeboxControl", args)?;
        Ok(serde_json::from_value(res)?)
    }

    fn send_action(&mut self, action: &str) -> Result<JukeboxStatus> {
        self.send_action_with(action, None, vec![])
    }

    pub fn playlist(&mut self) -> Result<JukeboxPlaylist> {
        let res = self.client.get("jukeboxControl", Query::with("action", "get"))?;
        Ok(serde_json::from_value::<JukeboxPlaylist>(res)?)
    }

    pub fn status(&mut self) -> Result<JukeboxStatus> {
        self.send_action("status")
    }

    pub fn play(&mut self) -> Result<JukeboxStatus> {
        self.send_action("start")
    }

    pub fn stop(&mut self) -> Result<JukeboxStatus> {
        self.send_action("stop")
    }

    /// Moves the jukebox's currently playing song to the provided index (zero-indexed).
    ///
    /// Using an index outside the range of the jukebox playlist will play the last song in the
    /// playlist.
    pub fn skip_to(&mut self, n: usize) -> Result<JukeboxStatus> {
        self.send_action_with("skip", n, vec![])
    }

    pub fn add(&mut self, song: Song) -> Result<JukeboxStatus> {
        self.send_action_with("add", None, vec![song.id as usize])
    }

    pub fn add_id(&mut self, id: usize) -> Result<JukeboxStatus> {
        self.send_action_with("add", None, vec![id])
    }

    pub fn add_all(&mut self, songs: &[Song]) -> Result<JukeboxStatus> {
        self.send_action_with("add", None, songs.to_vec().iter().map(|s| s.id as usize)
            .collect())
    }

    pub fn add_all_ids(&mut self, ids: &[usize]) -> Result<JukeboxStatus> {
        self.send_action_with("add", None, ids.to_vec())
    }

    pub fn clear(&mut self) -> Result<JukeboxStatus> {
        self.send_action("clear")
    }

    pub fn remove(&mut self, song: Song) -> Result<JukeboxStatus> {
        self.send_action_with("remove", song.id as usize, vec![])
    }

    pub fn remove_id(&mut self, id: usize) -> Result<JukeboxStatus> {
        self.send_action_with("remove", id, vec![])
    }

    pub fn shuffle(&mut self) -> Result<JukeboxStatus> {
        self.send_action("shuffle")
    }

    pub fn set_volume(&mut self, volume: f32) -> Result<JukeboxStatus> {
        let args = Query::with("action", "setGain")
            .arg("gain", volume)
            .build();
        let res = self.client.get("jukeboxControl", args)?;
        Ok(serde_json::from_value(res)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_playlist() {
        let parsed = serde_json::from_str::<JukeboxPlaylist>(r#"{
            "currentIndex" : 0,
            "playing" : false,
            "gain" : 0.75,
            "position" : 0,
            "entry" : [ {
                "id" : "1887",
                "parent" : "1880",
                "isDir" : false,
                "title" : "トリコリコPLEASE!!",
                "album" : "トリコリコPLEASE!!",
                "artist" : "AZALEA",
                "track" : 1,
                "year" : 2016,
                "coverArt" : "1880",
                "size" : 33457239,
                "contentType" : "audio/flac",
                "suffix" : "flac",
                "transcodedContentType" : "audio/ogg",
                "transcodedSuffix" : "ogg",
                "duration" : 227,
                "bitRate" : 1090,
                "path" : "A/AZALEA/トリコリコPLEASE!!/01 トリコリコPLEASE!!.flac",
                "isVideo" : false,
                "playCount" : 34,
                "discNumber" : 1,
                "created" : "2018-01-01T10:30:10.000Z",
                "albumId" : "260",
                "artistId" : "147",
                "type" : "music"
            }, {
                "id" : "1888",
                "parent" : "1880",
                "isDir" : false,
                "title" : "ときめき分類学",
                "album" : "トリコリコPLEASE!!",
                "artist" : "AZALEA",
                "track" : 2,
                "year" : 2016,
                "coverArt" : "1880",
                "size" : 40146569,
                "contentType" : "audio/flac",
                "suffix" : "flac",
                "transcodedContentType" : "audio/ogg",
                "transcodedSuffix" : "ogg",
                "duration" : 291,
                "bitRate" : 1033,
                "path" : "A/AZALEA/トリコリコPLEASE!!/02 ときめき分類学.flac",
                "isVideo" : false,
                "playCount" : 14,
                "discNumber" : 1,
                "created" : "2018-01-01T10:30:10.000Z",
                "albumId" : "260",
                "artistId" : "147",
                "type" : "music"
            } ]
         }"#).unwrap();

        assert_eq!(parsed.songs.len(), 2);
        assert!(!parsed.status.playing);
        assert_eq!(parsed.status.volume, 0.75);
    }
}

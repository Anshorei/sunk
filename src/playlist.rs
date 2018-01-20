use sunk::Sunk;
use song::Song;
use error::*;
use json;
use macros::*;

#[derive(Debug)]
pub struct Playlist {
    id: u64,
    name: String,
    song_count: u64,
    duration: u64,
    cover: String,
}

impl Playlist {
    fn from(j: json::Value) -> Result<Playlist> {
        if !j.is_object() { return Err(Error::ParseError("not an object")) }

        Ok(Playlist {
            id: fetch!(j->id: as_str, u64),
            name: fetch!(j->name: as_str).into(),
            song_count: fetch!(j->songCount: as_u64),
            duration: fetch!(j->duration: as_u64),
            cover: fetch!(j->coverArt: as_str).into(),
        })
    }

    fn songs(&self, sunk: &mut Sunk) -> Result<Vec<Song>> {
        get_playlist_content(sunk, self.id)
    }
}

fn get_playlists(sunk: &mut Sunk, user: Option<String>) -> Vec<Playlist> {
    unimplemented!()
}

fn get_playlist(sunk: &mut Sunk, id: u64) -> Result<Playlist> {
    unimplemented!()
}

fn get_playlist_content(sunk: &mut Sunk, id: u64) -> Result<Vec<Song>> {
    let (_, res) = sunk.get("getPlaylist", vec![("id", id)])?;
    let mut list = vec![];
    for song in res.pointer("/subsonic-response/playlist/entry")
        .ok_or(Error::ParseError("no entries found"))?
        .as_array().ok_or(Error::ParseError("not an array"))?
    {
        list.push(Song::from(song)?);
    }
    Ok(list)
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_util::*;

    #[test]
    fn test_songs_from_playlist() {
        let raw = json!(
            {
                "id" : "1",
                "name" : "Sleep Hits",
                "owner" : "user",
                "public" : false,
                "songCount" : 32,
                "duration" : 8334,
                "created" : "2018-01-01T14:45:07.464Z",
                "changed" : "2018-01-01T14:45:07.478Z",
                "coverArt" : "pl-2"
            }
        );

        let parsed = Playlist::from(raw).unwrap();
        let auth = load_credentials().unwrap();
        let mut srv = Sunk::new(&auth.0, &auth.1, &auth.2).unwrap();
        let songs = parsed.songs(&mut srv).unwrap();
    }
}
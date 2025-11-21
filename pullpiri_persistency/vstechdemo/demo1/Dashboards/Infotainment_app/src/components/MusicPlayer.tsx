import { useState } from 'react';
import { Card } from './ui/card';
import { Music, Play, Pause, SkipBack, SkipForward, Volume2, Heart } from 'lucide-react';
import { Button } from './ui/button';
import { Slider } from './ui/slider';

const mockPlaylist = [
  { id: 1, title: 'Electric Dreams', artist: 'Synthwave Collective', duration: '3:45', cover: '🎵' },
  { id: 2, title: 'Highway Cruiser', artist: 'Neon Nights', duration: '4:12', cover: '🎸' },
  { id: 3, title: 'Future Drive', artist: 'Digital Horizon', duration: '3:28', cover: '🎹' },
  { id: 4, title: 'Midnight Run', artist: 'RetroWave', duration: '4:01', cover: '🎧' },
];

export function MusicPlayer() {
  const [isPlaying, setIsPlaying] = useState(true);
  const [currentTrack, setCurrentTrack] = useState(0);
  const [progress, setProgress] = useState([45]);
  const [volume, setVolume] = useState([70]);
  const [liked, setLiked] = useState(false);

  const track = mockPlaylist[currentTrack];

  const handleNext = () => {
    setCurrentTrack((prev) => (prev + 1) % mockPlaylist.length);
  };

  const handlePrevious = () => {
    setCurrentTrack((prev) => (prev - 1 + mockPlaylist.length) % mockPlaylist.length);
  };

  return (
    <Card className="p-6 bg-slate-800/50 border-slate-700">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <Music className="w-5 h-5 text-blue-400" />
          <h3 className="text-white">Music Player</h3>
        </div>
        {isPlaying && (
          <div className="flex items-center gap-2">
            <div className="w-2 h-2 rounded-full bg-green-500 animate-pulse" />
            <span className="text-green-400 text-sm">Playing</span>
          </div>
        )}
      </div>

      {/* Current Track Display */}
      <div className="mb-6">
        <div className="flex items-center gap-4 mb-4">
          {/* Album Art */}
          <div className="w-24 h-24 bg-gradient-to-br from-blue-600 to-purple-600 rounded-lg flex items-center justify-center shadow-lg">
            <span className="text-5xl">{track.cover}</span>
          </div>
          
          {/* Track Info */}
          <div className="flex-1">
            <h4 className="text-white mb-1">{track.title}</h4>
            <p className="text-slate-400 text-sm mb-2">{track.artist}</p>
            <div className="flex items-center gap-2">
              <span className="text-slate-500 text-sm">{track.duration}</span>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setLiked(!liked)}
                className="p-1 h-auto"
              >
                <Heart className={`w-4 h-4 ${liked ? 'fill-red-500 text-red-500' : 'text-slate-400'}`} />
              </Button>
            </div>
          </div>
        </div>

        {/* Progress Bar */}
        <div className="mb-2">
          <Slider
            value={progress}
            onValueChange={setProgress}
            max={100}
            step={1}
            className="w-full"
          />
        </div>
        <div className="flex justify-between text-slate-500 text-sm">
          <span>1:42</span>
          <span>{track.duration}</span>
        </div>
      </div>

      {/* Controls */}
      <div className="flex items-center justify-center gap-3 mb-6">
        <Button
          variant="ghost"
          size="sm"
          onClick={handlePrevious}
          className="text-slate-400 hover:text-white"
        >
          <SkipBack className="w-5 h-5" />
        </Button>
        <Button
          onClick={() => setIsPlaying(!isPlaying)}
          className="bg-blue-600 hover:bg-blue-700 w-12 h-12 rounded-full"
        >
          {isPlaying ? (
            <Pause className="w-6 h-6" />
          ) : (
            <Play className="w-6 h-6" />
          )}
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onClick={handleNext}
          className="text-slate-400 hover:text-white"
        >
          <SkipForward className="w-5 h-5" />
        </Button>
      </div>

      {/* Volume Control */}
      <div className="flex items-center gap-3 mb-4">
        <Volume2 className="w-5 h-5 text-slate-400" />
        <Slider
          value={volume}
          onValueChange={setVolume}
          max={100}
          step={1}
          className="flex-1"
        />
        <span className="text-slate-400 text-sm w-10 text-right">{volume}%</span>
      </div>

      {/* Playlist */}
      <div className="border-t border-slate-700 pt-4">
        <p className="text-slate-400 text-sm mb-2">Up Next</p>
        <div className="space-y-2 max-h-32 overflow-y-auto">
          {mockPlaylist.map((item, index) => (
            <div
              key={item.id}
              onClick={() => setCurrentTrack(index)}
              className={`flex items-center gap-3 p-2 rounded cursor-pointer transition-colors ${
                index === currentTrack
                  ? 'bg-blue-950/30 border border-blue-600/50'
                  : 'bg-slate-900/50 hover:bg-slate-700/50'
              }`}
            >
              <span className="text-2xl">{item.cover}</span>
              <div className="flex-1 min-w-0">
                <p className={`text-sm truncate ${
                  index === currentTrack ? 'text-blue-400' : 'text-white'
                }`}>
                  {item.title}
                </p>
                <p className="text-slate-400 text-xs truncate">{item.artist}</p>
              </div>
              <span className="text-slate-500 text-xs">{item.duration}</span>
            </div>
          ))}
        </div>
      </div>
    </Card>
  );
}

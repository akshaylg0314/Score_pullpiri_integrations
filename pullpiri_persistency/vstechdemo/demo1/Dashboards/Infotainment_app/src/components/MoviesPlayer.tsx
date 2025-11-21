import { useState } from 'react';
import { Card } from './ui/card';
import { Play, Pause, FastForward, Rewind, Volume2, Film } from 'lucide-react';

const movies = [
  {
    id: 1,
    title: "Blade Runner 2049",
    genre: "Sci-Fi",
    duration: "2h 44m",
    rating: "8.0",
    poster: "🎬",
    year: "2017"
  },
  {
    id: 2,
    title: "The Matrix",
    genre: "Action",
    duration: "2h 16m",
    rating: "8.7",
    poster: "🔮",
    year: "1999"
  },
  {
    id: 3,
    title: "Interstellar",
    genre: "Drama",
    duration: "2h 49m",
    rating: "8.6",
    poster: "🌌",
    year: "2014"
  },
  {
    id: 4,
    title: "Avatar",
    genre: "Adventure",
    duration: "2h 42m",
    rating: "7.8",
    poster: "🌿",
    year: "2009"
  }
];

export function MoviesPlayer() {
  const [selectedMovie, setSelectedMovie] = useState(movies[0]);
  const [isPlaying, setIsPlaying] = useState(false);
  const [currentTime, setCurrentTime] = useState("0:00");
  const [totalTime] = useState("2:44:00");

  const togglePlay = () => {
    setIsPlaying(!isPlaying);
  };

  return (
    <Card className="bg-slate-800/50 border-slate-700 p-6">
      <div className="flex items-center gap-2 mb-6">
        <Film className="w-6 h-6 text-purple-400" />
        <h3 className="text-white text-lg">Movies & Entertainment</h3>
      </div>

      <div className="space-y-6">
        {/* Current Movie Display */}
        <div className="bg-slate-900/50 p-4 rounded-lg">
          <div className="flex items-start gap-4 mb-4">
            <div className="text-4xl">{selectedMovie.poster}</div>
            <div className="flex-1">
              <h4 className="text-white text-lg mb-1">{selectedMovie.title}</h4>
              <p className="text-slate-400 text-sm mb-1">
                {selectedMovie.genre} • {selectedMovie.year} • {selectedMovie.duration}
              </p>
              <div className="flex items-center gap-2">
                <div className="bg-yellow-600 text-black text-xs px-2 py-1 rounded">
                  ⭐ {selectedMovie.rating}
                </div>
              </div>
            </div>
          </div>

          {/* Progress Bar */}
          <div className="mb-4">
            <div className="bg-slate-700 h-2 rounded-full overflow-hidden">
              <div className="bg-purple-500 h-full rounded-full" style={{ width: '25%' }}></div>
            </div>
            <div className="flex justify-between text-slate-400 text-sm mt-1">
              <span>{currentTime}</span>
              <span>{totalTime}</span>
            </div>
          </div>

          {/* Controls */}
          <div className="flex items-center justify-center gap-4">
            <button className="p-2 text-slate-400 hover:text-white transition-colors">
              <Rewind className="w-6 h-6" />
            </button>
            
            <button 
              onClick={togglePlay}
              className="p-3 bg-purple-600 text-white rounded-full hover:bg-purple-700 transition-colors"
            >
              {isPlaying ? (
                <Pause className="w-6 h-6" />
              ) : (
                <Play className="w-6 h-6 ml-1" />
              )}
            </button>
            
            <button className="p-2 text-slate-400 hover:text-white transition-colors">
              <FastForward className="w-6 h-6" />
            </button>
            
            <button className="p-2 text-slate-400 hover:text-white transition-colors">
              <Volume2 className="w-5 h-5" />
            </button>
          </div>
        </div>

        {/* Movie List */}
        <div className="space-y-2">
          <p className="text-slate-400 text-sm mb-3">Recently Downloaded</p>
          {movies.map((movie) => (
            <div
              key={movie.id}
              onClick={() => setSelectedMovie(movie)}
              className={`p-3 rounded-lg cursor-pointer transition-colors ${
                selectedMovie.id === movie.id
                  ? 'bg-purple-600/20 border border-purple-600/30'
                  : 'bg-slate-900/30 hover:bg-slate-700/30'
              }`}
            >
              <div className="flex items-center gap-3">
                <div className="text-xl">{movie.poster}</div>
                <div className="flex-1 min-w-0">
                  <p className="text-white text-sm truncate">{movie.title}</p>
                  <p className="text-slate-400 text-xs">
                    {movie.genre} • {movie.duration}
                  </p>
                </div>
                <div className="text-yellow-400 text-xs">
                  ⭐ {movie.rating}
                </div>
              </div>
            </div>
          ))}
        </div>

        {/* Quick Stats */}
        <div className="grid grid-cols-3 gap-2 text-center">
          <div className="bg-slate-900/50 p-2 rounded">
            <p className="text-purple-400 text-lg">4</p>
            <p className="text-slate-400 text-xs">Downloaded</p>
          </div>
          <div className="bg-slate-900/50 p-2 rounded">
            <p className="text-green-400 text-lg">2.1GB</p>
            <p className="text-slate-400 text-xs">Storage Used</p>
          </div>
          <div className="bg-slate-900/50 p-2 rounded">
            <p className="text-blue-400 text-lg">8.2</p>
            <p className="text-slate-400 text-xs">Avg Rating</p>
          </div>
        </div>
      </div>
    </Card>
  );
}
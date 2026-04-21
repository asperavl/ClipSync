import { Filter, CloudUpload, Play, Star, Gamepad2 } from 'lucide-react';
import { ClipMetadata } from '../App';
import { convertFileSrc } from '@tauri-apps/api/core';

interface ClipLibraryProps {
  clips: ClipMetadata[];
  onSelectClip: (id: string) => void;
}

export function ClipLibrary({ clips, onSelectClip }: ClipLibraryProps) {
  // Format seconds to mm:ss
  const formatDuration = (secs: number) => {
    const m = Math.floor(secs / 60);
    const s = secs % 60;
    return `${m}:${s.toString().padStart(2, '0')}`;
  };

  // Format timestamp to "Today, 14:32" or "Yesterday" or date
  const formatTime = (ts: number) => {
    const d = new Date(ts * 1000);
    const now = new Date();
    const isToday = d.getDate() === now.getDate() && d.getMonth() === now.getMonth() && d.getFullYear() === now.getFullYear();
    if (isToday) {
      return `Today, ${d.getHours().toString().padStart(2, '0')}:${d.getMinutes().toString().padStart(2, '0')}`;
    }
    return d.toLocaleDateString();
  };

  return (
    <main className="flex-1 md:ml-60 pt-16 px-4 md:px-8 lg:px-12 pb-24 overflow-y-auto">
      <div className="flex justify-between items-end mb-6">
        <div>
          <h1 className="text-[40px] leading-[48px] tracking-[-0.02em] font-extrabold text-[#e5e2e1] mb-2">
            Recent Captures
          </h1>
          <p className="text-sm text-[#cfc2d6]">Sorted by date recorded</p>
        </div>
        
        {/* Filter/Sort Controls */}
        <div className="flex gap-3">
          <button className="bg-[#2a2a2a] border border-[#4d4354]/30 text-[#e5e2e1] px-4 py-2 rounded-full text-[12px] font-bold tracking-wider flex items-center gap-2 hover:bg-[#353534] transition-colors">
            <Filter className="w-4 h-4" />
            Filters
          </button>
          <button className="bg-[#b76dff] text-[#400071] px-4 py-2 rounded-full text-[12px] font-bold tracking-wider flex items-center gap-2 hover:opacity-90 transition-opacity shadow-[0_0_15px_rgba(183,109,255,0.2)]">
            <CloudUpload className="w-4 h-4" />
            Sync All
          </button>
        </div>
      </div>

      {/* Clip Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
        {clips.length === 0 ? (
          <div className="col-span-full py-20 flex flex-col items-center justify-center text-[#4d4354]">
            <Gamepad2 className="w-16 h-16 mb-4 opacity-50" />
            <p className="text-sm font-bold tracking-widest uppercase">No clips recorded yet</p>
            <p className="text-xs mt-2 text-center opacity-80">Press Ctrl+F9 while in-game to save the last 30 seconds</p>
          </div>
        ) : (
          clips.map((clip) => (
            <div 
              key={clip.id} 
              onClick={() => onSelectClip(clip.id)}
              className="group relative bg-[#ffffff]/5 border border-white/10 rounded-xl overflow-hidden hover:scale-[1.02] hover:bg-white/10 hover:shadow-[0_0_20px_rgba(168,85,247,0.15)] transition-all duration-300 flex flex-col cursor-pointer backdrop-blur-sm"
            >
              <div className="relative aspect-video overflow-hidden bg-[#201f1f]">
                {clip.thumbnail_path ? (
                  <img 
                    src={convertFileSrc(clip.thumbnail_path)} 
                    alt={clip.title}
                    className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500"
                  />
                ) : (
                  <div className="w-full h-full flex items-center justify-center">
                    <Gamepad2 className="w-12 h-12 text-[#4d4354] opacity-50" />
                  </div>
                )}
                <div className="absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-transparent opacity-60"></div>
                
                {/* Duration Pill */}
                <div className="absolute top-2 right-2 bg-black/70 backdrop-blur-md border border-white/20 text-[#e5e2e1] font-mono text-[11px] font-medium tracking-widest px-2 py-1 rounded">
                  {formatDuration(clip.duration_secs)}
                </div>

                {/* Favorite Badge */}
                {clip.is_favorite && (
                  <div className="absolute top-2 left-2 flex gap-1">
                    <span className="font-mono text-[#ddb7ff] text-[9px] font-medium tracking-widest bg-[#ddb7ff]/20 backdrop-blur border border-[#ddb7ff]/30 px-1.5 py-0.5 rounded uppercase flex items-center gap-1">
                      <Star className="w-3 h-3 fill-current" /> Favorite
                    </span>
                  </div>
                )}

                {/* Hover Overlay Play Button */}
                <div className="absolute inset-0 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity duration-300">
                  <div className="w-12 h-12 bg-[#ddb7ff]/20 backdrop-blur-md border border-[#ddb7ff]/50 rounded-full flex items-center justify-center text-[#ddb7ff] shadow-[0_0_15px_rgba(221,183,255,0.3)]">
                    <Play className="w-6 h-6 fill-current ml-1" />
                  </div>
                </div>
              </div>
              
              <div className="p-3 flex justify-between items-start gap-2">
                <div className="flex-1 min-w-0">
                  <h3 className="text-[12px] font-bold tracking-[0.05em] text-[#e5e2e1] truncate mb-1 group-hover:text-[#ddb7ff] transition-colors">
                    {clip.title}
                  </h3>
                  <div className="flex items-center gap-2">
                    <span className="font-mono text-[10px] font-medium tracking-widest text-[#cfc2d6]">{formatTime(clip.date_recorded)}</span>
                    {clip.game_name && (
                      <>
                        <span className="w-1 h-1 rounded-full bg-white/20"></span>
                        <span className="font-mono text-[10px] font-medium tracking-widest text-[#b9f231] bg-[#b9f231]/10 px-1.5 py-0.5 rounded truncate max-w-[80px]">{clip.game_name}</span>
                      </>
                    )}
                  </div>
                </div>
                <button 
                  aria-label="Favorite" 
                  className="text-zinc-500 hover:text-yellow-400 transition-colors p-1 z-10"
                  onClick={(e) => {
                    e.stopPropagation();
                    // Toggle favorite logic (requires invoke call to backend eventually)
                  }}
                >
                  <Star className={`w-[18px] h-[18px] ${clip.is_favorite ? 'text-yellow-400 fill-current' : ''}`} />
                </button>
              </div>
            </div>
          ))
        )}
      </div>

      <div className="h-24"></div>
    </main>
  );
}

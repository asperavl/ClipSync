import { ArrowLeft, Heart, Edit2, HardDrive, Maximize, Gauge, CloudUpload, Trash2 } from 'lucide-react';
import { ClipMetadata } from '../App';
import { convertFileSrc } from '@tauri-apps/api/core';

interface ClipDetailProps {
  clip: ClipMetadata;
  onBack: () => void;
}

export function ClipDetail({ clip, onBack }: ClipDetailProps) {
  return (
    <main className="flex-1 md:ml-60 pt-12 h-screen flex bg-[#000000] relative">
      {/* Left Area: Video Player (70%) */}
      <section className="w-full md:w-[70%] h-full relative group bg-black flex flex-col items-center justify-center">
        <video 
          controls 
          autoPlay 
          src={convertFileSrc(clip.file_path)} 
          poster={clip.thumbnail_path ? convertFileSrc(clip.thumbnail_path) : undefined}
          className="max-w-full max-h-full outline-none"
        />

        {/* Sync Indicator (only if status is syncing) */}
        {clip.cloud_status === 'syncing' && (
          <div className="absolute top-6 right-6 flex items-center gap-2 bg-black/60 backdrop-blur-md px-3 py-1.5 rounded-full border border-white/5 pointer-events-none">
            <div className="w-3 h-3 rounded-full animate-spin border-2 border-transparent border-t-[#ddb7ff] border-r-[#00daf3]"></div>
            <span className="font-mono text-[11px] font-medium text-[#cfc2d6]">SYNCING</span>
          </div>
        )}
      </section>

      {/* Right Panel: Metadata Sidebar (30%) */}
      <aside className="hidden md:flex w-[30%] h-full bg-[#121212]/90 backdrop-blur-3xl border-l border-white/10 flex-col overflow-y-auto">
        {/* Header */}
        <div className="p-6 border-b border-white/10 flex items-center justify-between">
          <button onClick={onBack} className="flex items-center gap-2 text-[#cfc2d6] hover:text-[#e5e2e1] transition-colors">
            <ArrowLeft className="w-[18px] h-[18px]" />
            <span className="text-[12px] font-bold">BACK</span>
          </button>
          <button className={`w-9 h-9 rounded-full bg-[#201f1f] flex items-center justify-center border border-white/10 hover:bg-[#2a2a2a] transition-all group ${clip.is_favorite ? 'text-yellow-400' : 'text-[#ddb7ff]'}`}>
            <Heart className={`w-[20px] h-[20px] group-hover:scale-110 transition-transform ${clip.is_favorite ? 'fill-current' : ''}`} />
          </button>
        </div>

        <div className="p-6 flex flex-col gap-6 flex-1">
          {/* Input Group: Title */}
          <div className="flex flex-col gap-3">
            <label className="font-mono text-[11px] font-medium text-[#cfc2d6]">CLIP TITLE</label>
            <div className="relative">
              <input 
                type="text" 
                defaultValue={clip.title}
                className="w-full bg-[#000000] border border-white/10 rounded-lg px-4 py-3 text-base text-[#e5e2e1] focus:border-[#ddb7ff] focus:ring-1 focus:ring-[#ddb7ff] outline-none transition-all shadow-inner" 
              />
              <Edit2 className="absolute right-3 top-1/2 -translate-y-1/2 text-[#4d4354] w-[18px] h-[18px]" />
            </div>
          </div>

          {/* Input Group: Game Name */}
          <div className="flex flex-col gap-3">
            <label className="font-mono text-[11px] font-medium text-[#cfc2d6]">GAME</label>
            <div className="relative">
              <div className="absolute left-3 top-1/2 -translate-y-1/2 w-6 h-6 rounded bg-[#201f1f] flex items-center justify-center overflow-hidden">
                <span className="text-[10px] text-white">🎮</span>
              </div>
              <input 
                type="text" 
                defaultValue={clip.game_name || ''}
                placeholder="Unknown Game"
                className="w-full bg-[#000000] border border-white/10 rounded-lg pl-12 pr-4 py-3 text-base text-[#e5e2e1] focus:border-[#ddb7ff] focus:ring-1 focus:ring-[#ddb7ff] outline-none transition-all shadow-inner" 
              />
            </div>
          </div>

          <hr className="border-white/10 my-2" />

          {/* Stats Bento Grid */}
          <div className="flex flex-col gap-3">
            <label className="font-mono text-[11px] font-medium text-[#cfc2d6]">CLIP METADATA</label>
            <div className="grid grid-cols-3 gap-1">
              <div className="bg-white/[0.03] border border-white/10 rounded-lg p-3 flex flex-col items-center justify-center gap-1 hover:bg-white/[0.05] transition-colors">
                <HardDrive className="w-[18px] h-[18px] text-[#cfc2d6]" />
                <span className="text-[12px] font-bold text-[#e5e2e1]">MP4</span>
              </div>
              <div className="bg-white/[0.03] border border-white/10 rounded-lg p-3 flex flex-col items-center justify-center gap-1 hover:bg-white/[0.05] transition-colors">
                <Maximize className="w-[18px] h-[18px] text-[#cfc2d6]" />
                <span className="text-[12px] font-bold text-[#e5e2e1]">HD</span>
              </div>
              <div className="bg-white/[0.03] border border-white/10 rounded-lg p-3 flex flex-col items-center justify-center gap-1 hover:bg-white/[0.05] transition-colors">
                <Gauge className="w-[18px] h-[18px] text-[#cfc2d6]" />
                <span className="text-[12px] font-bold text-[#e5e2e1]">{clip.duration_secs}s</span>
              </div>
            </div>
          </div>
        </div>

        {/* Bottom Actions */}
        <div className="p-6 border-t border-white/10 flex flex-col gap-3 mt-auto bg-[#131313]/50">
          <button className="w-full bg-[#842bd2] hover:bg-[#b76dff] text-white text-[12px] font-bold py-4 rounded-xl flex items-center justify-center gap-2 transition-all duration-300 shadow-[0_0_20px_rgba(132,43,210,0.3)] hover:shadow-[0_0_25px_rgba(132,43,210,0.5)]">
            <CloudUpload className="w-[20px] h-[20px]" />
            UPLOAD TO DRIVE
          </button>
          <button className="w-full bg-transparent border border-[#ffb4ab]/30 hover:border-[#ffb4ab] hover:bg-[#ffb4ab]/10 text-[#ffb4ab] text-[12px] font-bold py-3 rounded-xl flex items-center justify-center gap-2 transition-all duration-300">
            <Trash2 className="w-[18px] h-[18px]" />
            DELETE CLIP
          </button>
        </div>
      </aside>
    </main>
  );
}

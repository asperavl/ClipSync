import { FileVideo, Settings, HelpCircle } from 'lucide-react';

interface SidebarProps {
  currentView: 'library' | 'settings';
  setCurrentView: (view: 'library' | 'settings') => void;
  isRecording?: boolean;
}

export function Sidebar({ currentView, setCurrentView, isRecording = false }: SidebarProps) {
  return (
    <nav className="hidden md:flex fixed left-0 top-0 h-screen w-60 bg-[#121212]/80 backdrop-blur-2xl border-r border-white/10 shadow-[0px_0px_15px_rgba(168,85,247,0.15)] flex-col pt-16 pb-4 z-40">
      {/* Header */}
      <div className="px-6 mb-8 mt-2">
        <h1 className="text-purple-500 font-black text-xl italic font-headline-lg tracking-tight mb-1">
          ClipSync
        </h1>
        <p className="text-[#b9f231] font-mono text-[11px] font-medium tracking-widest opacity-80 uppercase">
          Pro Gamer Tier
        </p>
      </div>

      {/* Main Tabs */}
      <div className="flex-1 flex flex-col gap-2 w-full text-[12px] font-bold uppercase tracking-widest font-sans">
        <button
          onClick={() => setCurrentView('library')}
          className={`flex items-center gap-3 px-6 py-4 w-full transition-colors duration-200 group ${
            currentView === 'library'
              ? 'bg-purple-500/10 text-purple-500 border-r-2 border-purple-500'
              : 'text-zinc-500 hover:bg-white/5 hover:text-white'
          }`}
        >
          <FileVideo className={`w-5 h-5 ${currentView === 'library' ? '' : 'group-hover:scale-110 transition-transform'}`} />
          Library
        </button>

        <button
          onClick={() => setCurrentView('settings')}
          className={`flex items-center gap-3 px-6 py-4 w-full transition-colors duration-200 group ${
            currentView === 'settings'
              ? 'bg-purple-500/10 text-purple-500 border-r-2 border-purple-500'
              : 'text-zinc-500 hover:bg-white/5 hover:text-white'
          }`}
        >
          <Settings className={`w-5 h-5 ${currentView === 'settings' ? '' : 'group-hover:scale-110 transition-transform'}`} />
          Settings
        </button>
      </div>

      {/* CTA & Footer */}
      <div className="px-6 mt-auto flex flex-col gap-4">
        {isRecording && (
          <div className="flex items-center gap-2 bg-black/50 border border-white/10 rounded-lg p-3">
            <div className="w-2 h-2 rounded-full bg-red-500 animate-pulse"></div>
            <span className="font-mono text-[11px] font-medium tracking-widest text-[#e5e2e1] uppercase">
              Recording...
            </span>
          </div>
        )}
        <button className="text-zinc-500 flex items-center gap-3 py-4 hover:text-white transition-colors duration-200 text-[12px] font-bold uppercase tracking-widest">
          <HelpCircle className="w-5 h-5" />
          Support
        </button>
      </div>
    </nav>
  );
}

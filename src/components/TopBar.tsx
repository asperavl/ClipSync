import { Search, Minus, Square, X } from 'lucide-react';
import { getCurrentWindow } from '@tauri-apps/api/window';

export function TopBar() {
  const appWindow = getCurrentWindow();

  return (
    <header className="fixed z-50 bg-[#121212]/90 backdrop-blur-xl font-sans text-sm font-medium tracking-tight top-0 h-12 border-b border-white/10 flex items-center w-full select-none">
      <div data-tauri-drag-region className="w-4 h-full" />
      
      <div data-tauri-drag-region className="flex items-center gap-4 md:hidden">
        <div className="text-[#a855f7] font-black text-xl italic pointer-events-none">ClipSync</div>
      </div>
      
      <div data-tauri-drag-region className="h-full flex-1 md:flex-none md:w-[60%] flex items-center justify-center md:justify-start md:pl-64">
        <div className="flex items-center bg-white/5 rounded-md px-3 py-1.5 w-64 border border-white/10 focus-within:border-[#a855f7] transition-colors pointer-events-auto">
          <Search className="text-[#cfc2d6] w-4 h-4 mr-2" />
          <input 
            type="text" 
            placeholder="Search clips..." 
            className="bg-transparent border-none outline-none text-[#e5e2e1] text-[12px] font-bold tracking-wider w-full placeholder:text-[#4d4354]" 
          />
        </div>
      </div>

      <div data-tauri-drag-region className="flex-1 h-full" />

      <div className="flex items-center gap-2 pr-4">
        <button 
          onClick={() => appWindow.minimize()}
          aria-label="minimize" 
          className="w-10 h-8 flex items-center justify-center text-[#cfc2d6] hover:bg-white/10 hover:text-white transition-colors duration-200 rounded z-10"
        >
          <Minus className="w-4 h-4" />
        </button>
        <button 
          onClick={() => appWindow.toggleMaximize()}
          aria-label="maximize" 
          className="w-10 h-8 flex items-center justify-center text-[#cfc2d6] hover:bg-white/10 hover:text-white transition-colors duration-200 rounded z-10"
        >
          <Square className="w-3.5 h-3.5" />
        </button>
        <button 
          onClick={() => appWindow.hide()}
          aria-label="hide to tray" 
          className="w-10 h-8 flex items-center justify-center text-[#cfc2d6] hover:bg-[#ffb4ab] hover:text-[#410002] transition-colors duration-200 rounded z-10"
        >
          <X className="w-4 h-4" />
        </button>
      </div>
    </header>
  );
}

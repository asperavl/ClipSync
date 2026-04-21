import { Video, History, Mic, Keyboard, ChevronDown } from 'lucide-react';

export function Settings() {
  return (
    <main className="flex-1 md:ml-60 pt-16 md:pt-24 px-4 md:px-8 lg:px-12 pb-24 overflow-y-auto">
      <div className="mb-10 max-w-4xl mx-auto">
        <h2 className="text-[40px] leading-[48px] tracking-[-0.02em] font-extrabold text-[#e5e2e1] mb-2">Settings</h2>
        <p className="text-base text-[#cfc2d6] opacity-80">Configure your capture, audio, and hotkey preferences for optimal performance.</p>
      </div>

      <div className="max-w-4xl mx-auto grid grid-cols-1 md:grid-cols-2 gap-4">
        {/* Capture Settings */}
        <div className="bg-white/5 border border-white/10 rounded-xl p-5 flex flex-col gap-3 glass-panel">
          <div className="flex items-center gap-2 mb-2 border-b border-white/10 pb-4">
            <Video className="w-6 h-6 text-[#ddb7ff]" />
            <h3 className="text-2xl font-bold text-[#e5e2e1]">Capture</h3>
          </div>
          
          <div className="flex flex-col gap-2 mt-2">
            <label className="font-mono text-[11px] font-medium text-[#988d9f] uppercase tracking-widest">Resolution</label>
            <div className="relative">
              <select className="appearance-none w-full bg-black border border-white/10 rounded-lg py-3 px-4 text-sm text-[#e5e2e1] focus:ring-1 focus:ring-[#ddb7ff] focus:border-[#ddb7ff] transition-colors cursor-pointer outline-none shadow-[0_0_10px_rgba(221,183,255,0)] focus:shadow-[0_0_10px_rgba(221,183,255,0.3)]">
                <option value="1080p">1080p (FHD)</option>
                <option selected value="1440p">1440p (QHD)</option>
                <option value="2160p">2160p (4K)</option>
              </select>
              <div className="pointer-events-none absolute inset-y-0 right-0 flex items-center px-4 text-[#cfc2d6]">
                <ChevronDown className="w-4 h-4" />
              </div>
            </div>
          </div>

          <div className="flex flex-col gap-2 mt-4">
            <label className="font-mono text-[11px] font-medium text-[#988d9f] uppercase tracking-widest">Framerate (FPS)</label>
            <div className="relative">
              <select className="appearance-none w-full bg-black border border-white/10 rounded-lg py-3 px-4 text-sm text-[#e5e2e1] focus:ring-1 focus:ring-[#ddb7ff] focus:border-[#ddb7ff] transition-colors cursor-pointer outline-none shadow-[0_0_10px_rgba(221,183,255,0)] focus:shadow-[0_0_10px_rgba(221,183,255,0.3)]">
                <option value="30">30 FPS</option>
                <option selected value="60">60 FPS</option>
                <option value="120">120 FPS</option>
              </select>
              <div className="pointer-events-none absolute inset-y-0 right-0 flex items-center px-4 text-[#cfc2d6]">
                <ChevronDown className="w-4 h-4" />
              </div>
            </div>
          </div>
        </div>

        {/* Buffer Settings */}
        <div className="bg-white/5 border border-white/10 rounded-xl p-5 flex flex-col gap-3 glass-panel">
          <div className="flex items-center gap-2 mb-2 border-b border-white/10 pb-4">
            <History className="w-6 h-6 text-[#b9f231]" />
            <h3 className="text-2xl font-bold text-[#e5e2e1]">Buffer</h3>
          </div>
          <p className="text-sm text-[#cfc2d6] mb-4">Determine how much past gameplay is saved when you hit the clipping hotkey.</p>
          
          <div className="flex flex-col gap-4 mt-auto mb-4 relative">
            <div className="flex justify-between items-end">
              <label className="font-mono text-[11px] font-medium text-[#988d9f] uppercase tracking-widest">Duration</label>
              <span className="text-[12px] font-bold tracking-[0.05em] text-[#b9f231] bg-[#b9f231]/10 px-2 py-1 rounded">30s</span>
            </div>
            <div className="relative w-full h-6 flex items-center">
              <div className="absolute h-1 bg-[#b9f231] rounded-full top-1/2 -translate-y-1/2 left-0 w-1/2 pointer-events-none"></div>
              <input type="range" min="15" max="60" step="15" defaultValue="30" className="w-full relative z-10" />
            </div>
            <div className="flex justify-between font-mono text-[11px] font-medium text-[#988d9f] mt-1">
              <span>15s</span>
              <span>30s</span>
              <span>60s</span>
            </div>
          </div>
        </div>

        {/* Audio Settings */}
        <div className="bg-white/5 border border-white/10 rounded-xl p-5 flex flex-col gap-3 glass-panel">
          <div className="flex items-center gap-2 mb-2 border-b border-white/10 pb-4">
            <Mic className="w-6 h-6 text-[#00daf3]" />
            <h3 className="text-2xl font-bold text-[#e5e2e1]">Audio</h3>
          </div>
          
          <div className="flex items-center justify-between mt-4 p-3 rounded-lg bg-black/40 border border-white/5 hover:border-white/10 transition-colors">
            <div className="flex items-center gap-3">
              <div className="w-8 h-8 rounded bg-white/5 flex items-center justify-center text-[#cfc2d6]">
                <Mic className="w-4 h-4" />
              </div>
              <div>
                <div className="text-[12px] font-bold text-[#e5e2e1]">Desktop Audio</div>
                <div className="text-[12px] text-[#cfc2d6] mt-0.5">Capture game and system sounds</div>
              </div>
            </div>
            <div className="relative inline-block w-10 mr-2 align-middle select-none">
              <input type="checkbox" defaultChecked className="toggle-checkbox absolute block w-6 h-6 rounded-full bg-white border-4 appearance-none cursor-pointer opacity-0 z-10" id="toggle_desktop" />
              <label htmlFor="toggle_desktop" className="toggle-label block overflow-hidden h-6 rounded-full bg-gray-300 cursor-pointer"></label>
            </div>
          </div>

          <div className="flex items-center justify-between mt-2 p-3 rounded-lg bg-black/40 border border-white/5 hover:border-white/10 transition-colors">
            <div className="flex items-center gap-3">
              <div className="w-8 h-8 rounded bg-white/5 flex items-center justify-center text-[#cfc2d6]">
                <Mic className="w-4 h-4" />
              </div>
              <div>
                <div className="text-[12px] font-bold text-[#e5e2e1]">Microphone</div>
                <div className="text-[12px] text-[#cfc2d6] mt-0.5">Include your voice commentary</div>
              </div>
            </div>
            <div className="relative inline-block w-10 mr-2 align-middle select-none">
              <input type="checkbox" defaultChecked className="toggle-checkbox absolute block w-6 h-6 rounded-full bg-white border-4 appearance-none cursor-pointer opacity-0 z-10" id="toggle_mic" />
              <label htmlFor="toggle_mic" className="toggle-label block overflow-hidden h-6 rounded-full bg-gray-300 cursor-pointer"></label>
            </div>
          </div>
        </div>

        {/* Hotkeys */}
        <div className="bg-white/5 border border-white/10 rounded-xl p-5 flex flex-col gap-3 glass-panel">
          <div className="flex items-center gap-2 mb-2 border-b border-white/10 pb-4">
            <Keyboard className="w-6 h-6 text-[#b76dff]" />
            <h3 className="text-2xl font-bold text-[#e5e2e1]">Hotkeys</h3>
          </div>
          <div className="flex flex-col gap-4 mt-4">
            <div className="flex flex-col gap-2">
              <label className="font-mono text-[11px] font-medium text-[#988d9f] uppercase tracking-widest">Save Clip (Buffer)</label>
              <div className="relative flex items-center">
                <input 
                  type="text" 
                  readOnly 
                  value="Ctrl + F9" 
                  className="w-full bg-black/60 border border-white/10 rounded-lg py-3 px-4 text-[12px] font-bold text-[#e5e2e1] text-center tracking-wider focus:border-[#ddb7ff] focus:ring-1 focus:ring-[#ddb7ff] cursor-pointer transition-colors outline-none" 
                />
              </div>
              <p className="text-[12px] text-[#cfc2d6] text-center mt-1">Click to record a new key combination.</p>
            </div>
          </div>
          
          <div className="mt-auto pt-6 flex justify-end gap-4">
            <button className="px-4 py-2 rounded-lg border border-white/20 text-[#e5e2e1] hover:bg-white/5 transition-colors text-[12px] font-bold">Discard</button>
            <button className="px-6 py-2 rounded-lg bg-[#b76dff] text-[#400071] text-[12px] font-bold hover:opacity-90 transition-opacity shadow-[0_0_15px_rgba(183,109,255,0.4)]">Apply Changes</button>
          </div>
        </div>
      </div>
    </main>
  );
}

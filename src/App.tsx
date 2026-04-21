import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

import { TopBar } from "./components/TopBar";
import { Sidebar } from "./components/Sidebar";
import { ClipLibrary } from "./components/ClipLibrary";
import { Settings } from "./components/Settings";
import { ClipDetail } from "./components/ClipDetail";

export interface ClipMetadata {
  id: string;
  title: string;
  game_name: string | null;
  date_recorded: number;
  duration_secs: number;
  cloud_status: string;
  is_favorite: boolean;
  file_path: string;
  thumbnail_path: string;
}

function App() {
  const [currentView, setCurrentView] = useState<'library' | 'settings'>('library');
  const [selectedClipId, setSelectedClipId] = useState<string | null>(null);
  
  const [clips, setClips] = useState<ClipMetadata[]>([]);
  const [isRecording, setIsRecording] = useState(false);

  // Fetch initial data
  useEffect(() => {
    const fetchData = async () => {
      try {
        const fetchedClips = await invoke<ClipMetadata[]>('get_all_clips');
        setClips(fetchedClips.sort((a, b) => b.date_recorded - a.date_recorded));
        
        const status = await invoke<string>('get_status');
        setIsRecording(status === 'buffering' || status === 'saving');
      } catch (e) {
        console.error("Failed to fetch data:", e);
      }
    };
    
    fetchData();
    
    // Poll for status updates
    const interval = setInterval(async () => {
      try {
        const status = await invoke<string>('get_status');
        setIsRecording(status === 'buffering' || status === 'saving');
        
        // Also refresh clips if we are on the library view (in case a new clip was saved)
        const fetchedClips = await invoke<ClipMetadata[]>('get_all_clips');
        setClips(fetchedClips.sort((a, b) => b.date_recorded - a.date_recorded));
      } catch (e) {}
    }, 2000);
    
    return () => clearInterval(interval);
  }, []);

  const selectedClip = clips.find(c => c.id === selectedClipId);

  return (
    <div className="min-h-screen bg-[#000000] text-[#e5e2e1] font-sans flex flex-col md:flex-row antialiased overflow-hidden">
      <TopBar />
      
      {/* Sidebar is hidden when a clip is selected, to give full space to player */}
      {!selectedClipId && (
        <Sidebar currentView={currentView} setCurrentView={setCurrentView} isRecording={isRecording} />
      )}

      {selectedClipId && selectedClip ? (
        <ClipDetail clip={selectedClip} onBack={() => setSelectedClipId(null)} />
      ) : currentView === 'library' ? (
        <ClipLibrary clips={clips} onSelectClip={setSelectedClipId} />
      ) : (
        <Settings />
      )}
    </div>
  );
}

export default App;

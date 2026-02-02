"use client";

import { useState, useRef, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { 
  MessageSquare, 
  BrainCircuit, 
  User, 
  Quote, 
  ChevronDown, 
  ChevronUp,
  Radio
} from "lucide-react";

// Robust Interface matching page.tsx data shape
interface SimulationMsg {
  agent_id?: string | number;
  id?: string | number; 
  
  agent_role?: string;
  role?: string;
  
  agent_demographic?: string;
  demographic?: string;

  response?: string;
  thought_process?: string;
  category?: string;
  sentiment?: string;
  timestamp?: string;
}

interface FocusGroupChatProps {
  agents: SimulationMsg[];
}

export default function FocusGroupChat({ agents }: FocusGroupChatProps) {
  // Use string keys for set to handle composite IDs
  const [expandedThoughts, setExpandedThoughts] = useState<Set<string>>(new Set());
  const scrollRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom when new messages arrive
  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [agents.length]);

  const toggleThought = (uniqueKey: string) => {
    const newSet = new Set(expandedThoughts);
    if (newSet.has(uniqueKey)) {
      newSet.delete(uniqueKey);
    } else {
      newSet.add(uniqueKey);
    }
    setExpandedThoughts(newSet);
  };

  const getSentimentStyle = (s?: string) => {
    switch (s?.toLowerCase()) {
      case "positive": return "border-l-emerald-500 bg-emerald-50/30";
      case "negative": return "border-l-rose-500 bg-rose-50/30";
      default: return "border-l-slate-300 bg-white";
    }
  };

  return (
    <div className="flex flex-col h-[700px] bg-white rounded-xl border border-slate-200 overflow-hidden shadow-sm">
      
      {/* Header */}
      <div className="p-4 border-b border-slate-100 flex justify-between items-center bg-slate-50/50 backdrop-blur-sm z-10">
        <div className="flex items-center gap-3">
            <div className="relative">
                <div className="absolute inset-0 bg-indigo-400 rounded-full animate-ping opacity-20"></div>
                <div className="relative p-2 bg-indigo-100 text-indigo-600 rounded-lg">
                    <MessageSquare className="w-5 h-5" />
                </div>
            </div>
            <div>
                <h3 className="font-bold text-slate-800 text-sm">Focus Group Live Stream</h3>
                <div className="flex items-center gap-1.5">
                    <div className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />
                    <p className="text-[10px] text-slate-500 font-medium uppercase tracking-wide">Real-time Feed</p>
                </div>
            </div>
        </div>
        <div className="text-xs font-mono font-bold text-slate-400 bg-slate-100 px-3 py-1 rounded-full">
            {agents.length} Events
        </div>
      </div>

      {/* Chat Area */}
      <div 
        ref={scrollRef}
        className="flex-1 overflow-y-auto p-6 space-y-8 scroll-smooth"
      >
        <AnimatePresence mode="popLayout">
          {agents.map((msg, index) => {
            // --- CRITICAL FIX: COMPOSITE KEY GENERATION ---
            // Combine ID + Index to ensure every chat bubble has a unique DOM identity.
            const rawId = msg.agent_id || msg.id || "unknown";
            const uniqueKey = `${rawId}-${index}`; 

            const isThoughtOpen = expandedThoughts.has(uniqueKey);
            const isNegative = msg.sentiment === 'negative';
            const role = msg.agent_role || msg.role || "Participant";
            const demographic = msg.agent_demographic || msg.demographic || "Unknown";
            const response = msg.response || "";

            // Safety filter: Don't render empty shells
            if (!response && !msg.thought_process) return null;

            return (
              <motion.div
                key={uniqueKey} 
                initial={{ opacity: 0, y: 20, scale: 0.98 }}
                animate={{ opacity: 1, y: 0, scale: 1 }}
                layout
                className="group"
              >
                {/* Meta Header */}
                <div className="flex items-center gap-2 mb-2 ml-1 opacity-60 group-hover:opacity-100 transition-opacity">
                    <Radio className="w-3 h-3 text-slate-400" />
                    <span className="text-[10px] font-bold uppercase tracking-wider text-slate-500">
                        {msg.category || "Discussion"}
                    </span>
                    <span className="text-xs text-slate-300">•</span>
                    <span className="text-[10px] text-slate-400 font-mono">
                        {msg.timestamp ? new Date(msg.timestamp).toLocaleTimeString() : "Just now"}
                    </span>
                </div>

                {/* Message Card */}
                <div className={`relative rounded-xl border-l-4 shadow-sm p-5 transition-all duration-300 hover:shadow-md ${getSentimentStyle(msg.sentiment)} border-y border-r border-slate-200`}>
                    
                    {/* User Info Row */}
                    <div className="flex justify-between items-start mb-4">
                        <div className="flex items-center gap-3">
                            <div className={`w-9 h-9 rounded-full flex items-center justify-center text-sm font-bold shadow-sm border border-white ${isNegative ? 'bg-rose-100 text-rose-600' : 'bg-indigo-100 text-indigo-600'}`}>
                                <User className="w-4 h-4" />
                            </div>
                            <div>
                                <div className="font-bold text-slate-900 text-sm flex items-center gap-2">
                                    {role}
                                </div>
                                <div className="text-[11px] text-slate-500 font-medium max-w-[200px] leading-tight truncate">
                                    {demographic}
                                </div>
                            </div>
                        </div>

                        {/* Cognitive Toggle */}
                        {msg.thought_process && (
                            <button 
                                onClick={() => toggleThought(uniqueKey)}
                                className={`flex items-center gap-1.5 px-3 py-1.5 rounded-full text-[10px] font-bold transition-all border ${
                                    isThoughtOpen 
                                    ? 'bg-slate-800 text-white border-slate-800 shadow-lg' 
                                    : 'bg-white text-slate-500 border-slate-200 hover:border-indigo-300 hover:text-indigo-600'
                                }`}
                            >
                                <BrainCircuit className="w-3 h-3" />
                                {isThoughtOpen ? "Hide Logic" : "View Logic"}
                                {isThoughtOpen ? <ChevronUp className="w-3 h-3"/> : <ChevronDown className="w-3 h-3"/>}
                            </button>
                        )}
                    </div>

                    {/* Hidden Thought Process (The "Why") */}
                    <AnimatePresence>
                        {isThoughtOpen && msg.thought_process && (
                            <motion.div 
                                initial={{ height: 0, opacity: 0, marginBottom: 0 }}
                                animate={{ height: "auto", opacity: 1, marginBottom: 16 }}
                                exit={{ height: 0, opacity: 0, marginBottom: 0 }}
                                className="overflow-hidden"
                            >
                                <div className="p-4 bg-slate-900 rounded-lg border border-slate-700 shadow-inner relative overflow-hidden">
                                    <div className="absolute top-0 right-0 p-2 opacity-20">
                                        <BrainCircuit className="w-12 h-12 text-indigo-500" />
                                    </div>
                                    
                                    <div className="flex items-center gap-2 text-indigo-400 text-[10px] font-mono mb-2 uppercase tracking-widest border-b border-slate-700 pb-2">
                                        <span>System.Internal_Monologue</span>
                                    </div>
                                    <p className="text-slate-300 text-xs font-mono leading-relaxed">
                                        "{msg.thought_process}"
                                    </p>
                                </div>
                            </motion.div>
                        )}
                    </AnimatePresence>

                    {/* Spoken Response (The "What") */}
                    <div className="relative pl-4">
                        <div className="absolute left-0 top-1 bottom-1 w-0.5 bg-slate-200 rounded-full" />
                        <Quote className="w-4 h-4 text-slate-300 absolute -left-2 -top-2 bg-white p-0.5 rounded-full" />
                        <p className="text-sm text-slate-700 leading-relaxed font-medium">
                            {response}
                        </p>
                    </div>

                </div>
              </motion.div>
            );
          })}
        </AnimatePresence>

        {/* Empty State */}
        {agents.length === 0 && (
          <div className="h-full flex flex-col items-center justify-center text-slate-300">
             <div className="w-24 h-24 bg-slate-50 rounded-full flex items-center justify-center mb-6 border border-slate-100">
                <MessageSquare className="w-10 h-10 text-slate-300" />
             </div>
             <div className="font-bold tracking-widest uppercase text-xs text-slate-400">Briefing Room Empty</div>
             <div className="text-xs mt-2 text-slate-400">Waiting for participants to join the session...</div>
          </div>
        )}
      </div>
    </div>
  );
}
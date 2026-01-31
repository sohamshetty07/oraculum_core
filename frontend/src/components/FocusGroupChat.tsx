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
  AlertCircle
} from "lucide-react";

// Define the shape of the data coming from the API
interface SimulationMsg {
  agent_id: string | number;
  agent_role: string; // This is the Name (e.g., "Ravi")
  agent_demographic: string; // e.g., "Student, High Skepticism"
  response: string; // The spoken verdict
  thought_process?: string; // The hidden internal monologue
  category?: string; // e.g., "Round 1"
  sentiment?: string; // "positive", "negative", "neutral"
  timestamp?: string;
}

interface FocusGroupChatProps {
  agents: SimulationMsg[];
}

export default function FocusGroupChat({ agents }: FocusGroupChatProps) {
  // Track which thoughts are expanded
  const [expandedThoughts, setExpandedThoughts] = useState<Set<number>>(new Set());
  const scrollRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom when new messages arrive
  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [agents.length]);

  const toggleThought = (index: number) => {
    const newSet = new Set(expandedThoughts);
    if (newSet.has(index)) {
      newSet.delete(index);
    } else {
      newSet.add(index);
    }
    setExpandedThoughts(newSet);
  };

  const getSentimentStyle = (s?: string) => {
    switch (s?.toLowerCase()) {
      case "positive": return "border-l-emerald-500 bg-emerald-50/50";
      case "negative": return "border-l-rose-500 bg-rose-50/50";
      default: return "border-l-slate-300 bg-white";
    }
  };

  return (
    <div className="flex flex-col h-[700px] bg-slate-50 rounded-3xl border border-slate-200 overflow-hidden shadow-xl">
      {/* Header */}
      <div className="p-4 bg-white border-b border-slate-100 flex justify-between items-center shadow-sm z-10">
        <div className="flex items-center gap-3">
            <div className="p-2 bg-indigo-100 text-indigo-600 rounded-lg">
                <MessageSquare className="w-5 h-5" />
            </div>
            <div>
                <h3 className="font-bold text-slate-800 text-sm">Focus Group Live Stream</h3>
                <p className="text-xs text-slate-500">Real-time interaction log</p>
            </div>
        </div>
        <div className="text-xs font-mono text-slate-400">
            {agents.length} Events Captured
        </div>
      </div>

      {/* Chat Area */}
      <div 
        ref={scrollRef}
        className="flex-1 overflow-y-auto p-6 space-y-6 scroll-smooth"
      >
        <AnimatePresence mode="popLayout">
          {agents.map((msg, index) => {
            const isThoughtOpen = expandedThoughts.has(index);
            const isNegative = msg.sentiment === 'negative';
            const uniqueKey = `${msg.agent_id}-${index}`;

            return (
              <motion.div
                key={uniqueKey}
                initial={{ opacity: 0, y: 20, scale: 0.98 }}
                animate={{ opacity: 1, y: 0, scale: 1 }}
                layout
                className="group"
              >
                {/* Meta Header */}
                <div className="flex items-center gap-2 mb-2 ml-1">
                    <span className="text-[10px] font-bold uppercase tracking-wider text-slate-400 bg-slate-100 px-2 py-0.5 rounded-full">
                        {msg.category || "Discussion"}
                    </span>
                    <span className="text-xs text-slate-300">•</span>
                    <span className="text-xs text-slate-400 font-medium">
                        {msg.timestamp || "Just now"}
                    </span>
                </div>

                {/* Message Card */}
                <div className={`relative rounded-2xl border-l-4 shadow-sm p-5 transition-all duration-300 hover:shadow-md ${getSentimentStyle(msg.sentiment)} border-y border-r border-slate-200`}>
                    
                    {/* User Info */}
                    <div className="flex justify-between items-start mb-3">
                        <div className="flex items-center gap-3">
                            <div className={`w-10 h-10 rounded-full flex items-center justify-center text-sm font-bold shadow-inner ${isNegative ? 'bg-rose-100 text-rose-600' : 'bg-indigo-100 text-indigo-600'}`}>
                                <User className="w-5 h-5" />
                            </div>
                            <div>
                                <div className="font-bold text-slate-800 text-sm flex items-center gap-2">
                                    {msg.agent_role}
                                </div>
                                <div className="text-[11px] text-slate-500 max-w-[200px] leading-tight">
                                    {msg.agent_demographic}
                                </div>
                            </div>
                        </div>

                        {/* Cognitive Toggle (The Brain Button) */}
                        {msg.thought_process && (
                            <button 
                                onClick={() => toggleThought(index)}
                                className={`flex items-center gap-1.5 px-3 py-1.5 rounded-full text-[10px] font-bold transition-colors border ${
                                    isThoughtOpen 
                                    ? 'bg-indigo-600 text-white border-indigo-600' 
                                    : 'bg-white text-indigo-600 border-indigo-100 hover:bg-indigo-50'
                                }`}
                            >
                                <BrainCircuit className="w-3 h-3" />
                                {isThoughtOpen ? "Hide Thoughts" : "Reveal Thoughts"}
                                {isThoughtOpen ? <ChevronUp className="w-3 h-3"/> : <ChevronDown className="w-3 h-3"/>}
                            </button>
                        )}
                    </div>

                    {/* Hidden Thought Process (The "Why") */}
                    <AnimatePresence>
                        {isThoughtOpen && msg.thought_process && (
                            <motion.div 
                                initial={{ height: 0, opacity: 0 }}
                                animate={{ height: "auto", opacity: 1 }}
                                exit={{ height: 0, opacity: 0 }}
                                className="overflow-hidden"
                            >
                                <div className="mb-4 p-3 bg-slate-800 rounded-lg border border-slate-700 relative">
                                    <div className="absolute top-0 left-0 w-1 h-full bg-purple-500 rounded-l-lg" />
                                    <div className="flex items-center gap-2 text-purple-300 text-[10px] font-mono mb-1 uppercase tracking-widest">
                                        <BrainCircuit className="w-3 h-3" /> Internal Monologue
                                    </div>
                                    <p className="text-slate-300 text-xs font-mono leading-relaxed italic">
                                        "{msg.thought_process}"
                                    </p>
                                </div>
                            </motion.div>
                        )}
                    </AnimatePresence>

                    {/* Spoken Response (The "What") */}
                    <div className="relative pl-4">
                        <div className="absolute left-0 top-0 bottom-0 w-0.5 bg-slate-200 rounded-full" />
                        <Quote className="w-4 h-4 text-slate-300 absolute -left-2 -top-1 bg-white" />
                        <p className="text-sm text-slate-700 leading-relaxed font-medium">
                            {msg.response}
                        </p>
                    </div>

                </div>
              </motion.div>
            );
          })}
        </AnimatePresence>

        {/* Empty State */}
        {agents.length === 0 && (
          <div className="h-full flex flex-col items-center justify-center opacity-40">
             
             <div className="w-20 h-20 bg-slate-100 rounded-full flex items-center justify-center mb-4">
                <MessageSquare className="w-8 h-8 text-slate-400" />
             </div>
             <div className="font-bold tracking-widest uppercase text-sm text-slate-500">Briefing Room Empty</div>
             <div className="text-xs mt-2 text-slate-400">Waiting for participants to arrive...</div>
          </div>
        )}
      </div>
    </div>
  );
}
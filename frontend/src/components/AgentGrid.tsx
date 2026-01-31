"use client";

import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { User, MessageSquare, BrainCircuit, X, CheckCircle, BarChart3 } from "lucide-react";

// Define a robust interface that handles both Agent and Result data shapes
interface AgentData {
  agent_id?: string | number;
  id?: string | number; // Fallback from initial agent generation
  
  agent_role?: string;
  role?: string; // Fallback
  name?: string;

  agent_demographic?: string;
  demographic?: string; // Fallback

  response?: string;
  thought_process?: string; // <--- The hidden cognitive trace
  sentiment?: string;
  category?: string;
}

export default function AgentGrid({ agents }: { agents: AgentData[] }) {
  const [selectedAgent, setSelectedAgent] = useState<AgentData | null>(null);

  const getSentimentColor = (s?: string) => {
    if (s === 'positive') return 'text-emerald-700 bg-emerald-50 border-emerald-200';
    if (s === 'negative') return 'text-rose-700 bg-rose-50 border-rose-200';
    return 'text-amber-700 bg-amber-50 border-amber-200';
  };

  return (
    <>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
        <AnimatePresence mode="popLayout">
          {agents.map((agent, index) => {
            // Normalize data fields
            const id = agent.agent_id || agent.id;
            const role = agent.agent_role || agent.role || agent.name || "Agent";
            const demographic = agent.agent_demographic || agent.demographic || "";
            const sentiment = agent.sentiment || "neutral";

            return (
              <motion.div
                key={`${id}-${index}`}
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                layout
                onClick={() => agent.response && setSelectedAgent(agent)}
                className={`card-base rounded-xl p-5 relative group transition-all duration-300 ${agent.response ? 'cursor-pointer hover:border-[var(--primary)] hover:shadow-md' : 'opacity-80'}`}
              >
                {/* Header */}
                <div className="flex justify-between items-start mb-4">
                  <div className="flex items-center gap-3">
                    <div className="w-10 h-10 rounded-full bg-slate-100 flex items-center justify-center border border-slate-200 text-slate-500 group-hover:bg-indigo-50 group-hover:text-[var(--primary)] transition-colors">
                      <User className="w-5 h-5" />
                    </div>
                    <div>
                      <h3 className="font-bold text-sm text-gray-900 leading-tight">
                        {role}
                      </h3>
                      <p className="text-[10px] text-gray-500 font-medium uppercase tracking-wide truncate max-w-[140px] mt-0.5">
                        {demographic.split(',')[0]}
                      </p>
                    </div>
                  </div>
                  {agent.sentiment && (
                     <div className={`px-2 py-0.5 rounded border text-[10px] font-bold uppercase tracking-wider ${getSentimentColor(sentiment)}`}>
                      {sentiment}
                    </div>
                  )}
                </div>

                {/* Response Preview */}
                <div className="mb-8 relative min-h-[60px]">
                  {agent.response ? (
                    <>
                      <p className="text-sm text-gray-600 leading-relaxed line-clamp-3">
                        "{agent.response}"
                      </p>
                      {/* Visual Cue for Thoughts */}
                      {agent.thought_process && (
                          <div className="absolute -bottom-6 -right-1 opacity-0 group-hover:opacity-100 transition-opacity duration-200">
                               <span className="flex items-center gap-1.5 text-[10px] font-bold text-[var(--primary)] bg-indigo-50 px-2 py-1 rounded-full border border-indigo-100 shadow-sm">
                                  <BrainCircuit className="w-3 h-3" /> View Thoughts
                              </span>
                          </div>
                      )}
                    </>
                  ) : (
                    <div className="flex items-center gap-2 text-gray-400 text-sm italic animate-pulse">
                        <div className="w-2 h-2 bg-gray-400 rounded-full animate-bounce" />
                        Generating response...
                    </div>
                  )}
                </div>

                {/* Footer Meta */}
                <div className="absolute bottom-5 left-5 right-5 flex items-center justify-between text-[10px] text-gray-400 font-mono pt-3 border-t border-gray-100">
                  <div className="flex items-center gap-1.5">
                    <CheckCircle className="w-3 h-3" />
                    ID: {String(id).padStart(3, '0')}
                  </div>
                  <div className="uppercase tracking-wider font-semibold">
                    {(agent.category || "Processing")}
                  </div>
                </div>
              </motion.div>
            );
          })}
        </AnimatePresence>

        {agents.length === 0 && (
          <div className="col-span-full h-64 flex flex-col items-center justify-center border-2 border-dashed border-gray-200 rounded-xl bg-slate-50/50">
             <BarChart3 className="w-8 h-8 text-gray-300 mb-2" />
             <div className="font-bold text-gray-400 text-sm uppercase tracking-widest">Awaiting Simulation Data</div>
             <div className="text-xs text-gray-400 mt-1">Deploy the swarm to generate insights</div>
          </div>
       )}
      </div>

      {/* Modal for Details (Thought Reveal) */}
      <AnimatePresence>
        {selectedAgent && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 bg-gray-900/60 backdrop-blur-sm z-50 flex items-center justify-center p-4"
            onClick={() => setSelectedAgent(null)}
          >
            <motion.div
              initial={{ scale: 0.95, y: 10 }}
              animate={{ scale: 1, y: 0 }}
              exit={{ scale: 0.95, y: 10 }}
              onClick={(e) => e.stopPropagation()}
              className="bg-white rounded-xl shadow-2xl w-full max-w-lg overflow-hidden flex flex-col max-h-[85vh]"
            >
              {/* Modal Header */}
              <div className="p-5 border-b border-gray-100 bg-slate-50 flex justify-between items-center">
                <div className="flex items-center gap-3">
                    <div className="w-10 h-10 rounded-full bg-white border border-slate-200 flex items-center justify-center text-[var(--primary)] shadow-sm">
                         <User className="w-5 h-5" />
                    </div>
                    <div>
                        <h3 className="text-base font-bold text-gray-900">
                            {selectedAgent.agent_role || selectedAgent.role}
                        </h3>
                        <p className="text-xs text-gray-500">
                            {selectedAgent.agent_demographic || selectedAgent.demographic}
                        </p>
                    </div>
                </div>
                <button 
                  onClick={() => setSelectedAgent(null)}
                  className="p-2 hover:bg-gray-200 rounded-full transition-colors text-gray-500"
                >
                  <X className="w-5 h-5" />
                </button>
              </div>

              {/* Modal Content */}
              <div className="p-6 space-y-6 overflow-y-auto">
                {/* 1. The Verdict */}
                <div>
                    <h4 className="text-xs font-bold text-gray-400 uppercase tracking-widest mb-2 flex items-center gap-2">
                        <MessageSquare className="w-3 h-3" /> Spoken Response
                    </h4>
                    <div className="text-sm text-gray-800 leading-relaxed bg-gray-50 p-4 rounded-lg border border-gray-200">
                        "{selectedAgent.response}"
                    </div>
                </div>

                {/* 2. The Thought Process (If available) */}
                {selectedAgent.thought_process ? (
                    <div>
                        <h4 className="text-xs font-bold text-[var(--primary)] uppercase tracking-widest mb-2 flex items-center gap-2">
                            <BrainCircuit className="w-3 h-3" /> Cognitive Trace (Hidden)
                        </h4>
                        <div className="text-xs text-gray-600 leading-relaxed font-mono bg-indigo-50/50 p-4 rounded-lg border border-indigo-100 shadow-inner">
                            {selectedAgent.thought_process}
                        </div>
                    </div>
                ) : (
                    <div className="text-center py-6 border border-dashed border-gray-200 rounded-lg">
                        <p className="text-gray-400 text-xs italic">No internal thought process captured for this round.</p>
                    </div>
                )}
              </div>
              
              <div className="p-3 bg-gray-50 border-t border-gray-100 text-center text-[10px] text-gray-400 uppercase tracking-wider">
                Oraculum Memory Stream v3.0
              </div>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>
    </>
  );
}
"use client";

import { useState, useEffect, useRef } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { 
  Play, Activity, Users, Zap, Layers, FileText, Download, 
  Image as ImageIcon, X, Target, Sparkles, BookOpen, 
  BarChart3, Globe, ShieldCheck, ChevronDown, Loader2,
  BrainCircuit, CheckCircle2
} from "lucide-react";

import AgentGrid from "../components/AgentGrid";
import FocusGroupChat from "../components/FocusGroupChat";

// --- TYPES ---
interface Agent {
  // Common fields
  id?: number | string;
  agent_id?: number | string;
  
  name?: string;
  role?: string;
  agent_role?: string;
  
  demographic?: string;
  agent_demographic?: string;
  
  response?: string;
  thought_process?: string;
  sentiment?: string;
  category?: string;
}

export default function Home() {
  // --- CONFIGURATION STATE ---
  const [product, setProduct] = useState("Cadbury Zero Sugar Silk");
  const [scenarioType, setScenarioType] = useState("product_launch");
  const [context, setContext] = useState("High protein, zero sugar, tastes like real milk chocolate");
  const [count, setCount] = useState(5);
  const [image, setImage] = useState<string | null>(null); 
  const [pdf, setPdf] = useState<string | null>(null);
  
  // DEMOGRAPHIC MATRIX
  const [region, setRegion] = useState("Mumbai");
  const [ageMin, setAgeMin] = useState(18);
  const [ageMax, setAgeMax] = useState(35);
  const [gender, setGender] = useState("Mixed");
  const [sec, setSec] = useState("SEC A (Premium)");

  // --- SIMULATION ENGINE STATE ---
  const [status, setStatus] = useState<"idle" | "processing" | "completed">("idle"); 
  const [jobId, setJobId] = useState("");
  const [agents, setAgents] = useState<Agent[]>([]);
  const [progress, setProgress] = useState(0); // 0-100
  
  const [analyzing, setAnalyzing] = useState(false);
  const [report, setReport] = useState<string | null>(null);

  const fileInputRef = useRef<HTMLInputElement>(null);
  const pdfInputRef = useRef<HTMLInputElement>(null);

  // --- HELPER LOGIC ---
  const getConstructedAudience = () => {
    return `${region} based ${gender === 'Mixed' ? 'Consumers' : gender} aged ${ageMin}-${ageMax}, belonging to ${sec}`;
  };

  const getContextLabel = () => {
     switch (scenarioType) {
         case "focus_group": return "Discussion Topic";
         case "creative_test": return "Ad Copy / Variant B";
         case "cx_flow": return "Journey Context";
         default: return "Key Benefits / Context";
     }
  };

  // --- HANDLERS ---
  const handleFileUpload = (e: React.ChangeEvent<HTMLInputElement>, type: 'image' | 'pdf') => {
    const file = e.target.files?.[0];
    if (file) {
      const reader = new FileReader();
      reader.onloadend = () => {
        const raw = (reader.result as string).split(',')[1];
        if (type === 'image') setImage(raw);
        else setPdf(raw);
      };
      reader.readAsDataURL(file);
    }
  };

  const startSimulation = async () => {
    setStatus("processing");
    setAgents([]); 
    setReport(null);
    setProgress(5); // Initial bump
    
    try {
      const res = await fetch("http://127.0.0.1:8080/api/simulate", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          scenario: scenarioType,
          product_name: product,
          target_audience: getConstructedAudience(),
          context: context,
          agent_count: count,
          image_data: image,
          pdf_data: pdf
        }),
      });
      const data = await res.json();
      setJobId(data.job_id);
    } catch (e) {
      console.error("Failed to connect", e);
      setStatus("idle");
      alert("Failed to connect to Neural Engine. Is the backend running?");
    }
  };

  const generateReport = async () => {
    if (!jobId) return;
    setAnalyzing(true);
    try {
        const res = await fetch("http://127.0.0.1:8080/api/analyze", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ job_id: jobId }),
        });
        const data = await res.json();
        setReport(data.report);
    } catch (e) {
        console.error("Analysis failed", e);
    } finally {
        setAnalyzing(false);
    }
  };

  // --- POLLING LOOP ---
  useEffect(() => {
    if (!jobId || status === "completed") return;
    
    const interval = setInterval(async () => {
      try {
        const res = await fetch(`http://127.0.0.1:8080/api/status/${jobId}`);
        const data = await res.json();

        // Update Progress Bar
        if (data.progress) setProgress(Math.round(data.progress * 100));

        // Update Grid/Chat
        if (data.results && data.results.length > 0) {
            if (scenarioType === 'focus_group') {
                setAgents(data.results); // Focus group is a linear feed
            } else {
                // Parallel grid needs merging to prevent flickering
                setAgents(prev => {
                    const newMap = new Map(prev.map(a => [a.id || a.agent_id, a]));
                    data.results.forEach((r: any) => {
                         // Map 'agent_id' from result to 'id' for Agent interface
                         const id = r.agent_id || r.id;
                         newMap.set(id, { ...newMap.get(id), ...r, id });
                    });
                    return Array.from(newMap.values()) as Agent[];
                });
            }
        } 
        // Initial Recruitment Phase (Before responses)
        else if (data.agents && agents.length === 0) {
            setAgents(data.agents);
        }

        if (data.status === "completed") {
          setStatus("completed");
          setProgress(100);
          clearInterval(interval);
        }
      } catch (e) {
          console.warn("Polling hiccup", e);
      }
    }, 1000); 
    return () => clearInterval(interval);
  }, [jobId, status, agents.length, scenarioType]); 

  // --- DOWNLOAD ---
  const downloadReport = () => {
    if (agents.length === 0) return;
    const headers = ["ID", "Category/Round", "Role", "Demographic", "Verdict", "Thought Process"];
    
    const rows = agents.map(a => {
        // ROBUST DATA MAPPING (Handles both Focus Group & Grid formats)
        const id = a.agent_id || a.id || "N/A";
        const category = a.category || "General";
        
        // Focus group results use 'agent_role', Grid uses 'role' or 'name'
        const role = a.agent_role || a.role || a.name || "Participant";
        const demo = a.agent_demographic || a.demographic || "";
        
        const response = (a.response || "").replace(/"/g, '""'); // Escape CSV quotes
        const thoughts = (a.thought_process || "").replace(/"/g, '""');

        return [
            id,
            `"${category}"`, 
            `"${role}"`, 
            `"${demo}"`,
            `"${response}"`,
            `"${thoughts}"` 
        ].join(",");
    });

    const csvContent = [headers.join(","), ...rows].join("\n");
    const blob = new Blob([csvContent], { type: "text/csv;charset=utf-8;" });
    const link = document.createElement("a");
    link.href = URL.createObjectURL(blob);
    link.download = `ORACULUM_${scenarioType}_${new Date().toISOString().slice(0,10)}.csv`;
    link.click();
  };

  // Stats
  const positiveCount = agents.filter(a => a.sentiment === 'positive').length;
  const positivePct = agents.length ? Math.round((positiveCount / agents.length) * 100) : 0;

  return (
    <main className="min-h-screen bg-slate-50 font-sans text-slate-900 flex flex-col">
      
      {/* 1. TOP NAVIGATION BAR */}
      <header className="bg-white border-b border-slate-200 sticky top-0 z-30 shadow-sm">
        <div className="max-w-[1600px] mx-auto px-6 h-16 flex items-center justify-between">
            <div className="flex items-center gap-3">
                <div className="bg-indigo-600 p-2 rounded-lg text-white">
                    <Globe className="w-5 h-5" />
                </div>
                <div>
                    <h1 className="text-lg font-bold tracking-tight text-slate-900 leading-none">Oraculum <span className="text-slate-400 font-light">Core</span></h1>
                    <p className="text-[10px] text-slate-500 font-medium uppercase tracking-wider">Cognitive Market Intelligence v4.0</p>
                </div>
            </div>

            {/* STATUS PILL */}
            <div className="hidden md:flex items-center gap-4">
                {status !== 'idle' && (
                    <div className="flex flex-col items-end mr-4">
                        <div className="flex items-center gap-2 text-xs font-semibold text-slate-600 mb-1">
                            {status === 'processing' && <Loader2 className="w-3 h-3 animate-spin text-indigo-600" />}
                            {status === 'processing' ? 'SIMULATION ACTIVE' : 'COMPLETE'}
                        </div>
                        <div className="w-32 h-1.5 bg-slate-100 rounded-full overflow-hidden">
                            <motion.div 
                                className="h-full bg-indigo-600 rounded-full"
                                initial={{ width: 0 }}
                                animate={{ width: `${progress}%` }}
                            />
                        </div>
                    </div>
                )}
                
                <div className="h-8 px-3 rounded-md bg-slate-100 border border-slate-200 flex items-center gap-2 text-[10px] font-bold text-slate-600 uppercase tracking-wide">
                    <div className={`w-1.5 h-1.5 rounded-full ${status === 'processing' ? 'bg-emerald-500 animate-pulse' : 'bg-slate-400'}`} />
                    M4 Neural Engine
                </div>
            </div>
        </div>
      </header>

      {/* 2. MAIN WORKSPACE */}
      <div className="flex-1 max-w-[1600px] mx-auto w-full p-6 grid grid-cols-1 lg:grid-cols-12 gap-8">
        
        {/* LEFT: MISSION CONTROL */}
        <div className="lg:col-span-4 space-y-6">
          
          <div className="bg-white rounded-xl shadow-sm border border-slate-200 overflow-hidden">
            <div className="p-4 border-b border-slate-100 bg-slate-50/50 flex items-center gap-2">
                <Zap className="w-4 h-4 text-indigo-600" />
                <h2 className="text-xs font-bold text-slate-700 uppercase tracking-widest">Configuration</h2>
            </div>

            <div className="p-5 space-y-6">
              
              {/* SCENARIO SELECTOR */}
              <div className="space-y-2">
                <label className="text-[11px] font-bold text-slate-500 uppercase tracking-wide">Simulation Protocol</label>
                <div className="relative">
                  <select 
                    value={scenarioType}
                    onChange={(e) => setScenarioType(e.target.value)}
                    className="w-full appearance-none bg-white border border-slate-200 text-slate-700 text-sm rounded-lg p-3 pr-8 focus:ring-2 focus:ring-indigo-500 focus:border-transparent outline-none transition-shadow font-medium"
                  >
                    <option value="product_launch">Product Launch Reaction</option>
                    <option value="focus_group">Multi-Agent Focus Group (Debate)</option>
                    <option value="ab_messaging">Messaging Strategy A/B</option>
                    <option value="creative_test">Creative Asset Test</option>
                    <option value="cx_flow">Customer Journey Map</option>
                  </select>
                  <ChevronDown className="absolute right-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-400 pointer-events-none" />
                </div>
              </div>

              {/* DEMOGRAPHIC MATRIX */}
              <div className="bg-slate-50 rounded-lg p-4 border border-slate-200 space-y-4">
                <div className="flex items-center gap-2 text-[11px] font-bold text-slate-700 uppercase">
                   <Target className="w-3 h-3 text-indigo-600" /> Target Matrix
                </div>
                
                {/* Region & Class */}
                <div className="grid grid-cols-2 gap-3">
                    <div>
                        <span className="text-[10px] font-bold text-slate-400 uppercase">Region</span>
                        <input 
                            type="text" value={region} onChange={(e) => setRegion(e.target.value)} 
                            className="w-full mt-1 bg-white border border-slate-200 rounded-md p-2 text-xs font-medium focus:ring-1 focus:ring-indigo-500 outline-none" 
                        />
                    </div>
                    <div>
                        <span className="text-[10px] font-bold text-slate-400 uppercase">SEC Class</span>
                        <select value={sec} onChange={(e) => setSec(e.target.value)} className="w-full mt-1 bg-white border border-slate-200 rounded-md p-2 text-xs font-medium outline-none">
                            <option>SEC A (Premium)</option>
                            <option>SEC B (Middle)</option>
                            <option>SEC C (Mass)</option>
                        </select>
                    </div>
                </div>

                {/* Age Range */}
                <div className="space-y-1">
                     <span className="text-[10px] font-bold text-slate-400 uppercase">Age Range: {ageMin} - {ageMax}</span>
                     <div className="flex gap-2 items-center">
                        <input type="range" min="18" max="60" value={ageMin} onChange={(e) => setAgeMin(Number(e.target.value))} className="w-full h-1 bg-slate-200 rounded-lg appearance-none cursor-pointer" />
                        <input type="range" min="18" max="60" value={ageMax} onChange={(e) => setAgeMax(Number(e.target.value))} className="w-full h-1 bg-slate-200 rounded-lg appearance-none cursor-pointer" />
                     </div>
                </div>

                {/* Calculated Audience */}
                <div className="pt-3 border-t border-slate-200">
                    <div className="flex items-start gap-2">
                        <CheckCircle2 className="w-3 h-3 text-emerald-500 mt-0.5" />
                        <p className="text-xs text-slate-600 font-medium leading-tight">{getConstructedAudience()}</p>
                    </div>
                </div>
              </div>

               {/* PRODUCT CONTEXT */}
               <div className="space-y-3">
                 <div>
                    <label className="text-[11px] font-bold text-slate-500 uppercase tracking-wide">Product / Brand</label>
                    <input type="text" value={product} onChange={(e) => setProduct(e.target.value)} className="w-full mt-1 bg-white border border-slate-200 rounded-lg p-3 text-sm font-medium focus:ring-2 focus:ring-indigo-500 outline-none" />
                </div>
                <div>
                    <label className="text-[11px] font-bold text-slate-500 uppercase tracking-wide">{getContextLabel()}</label>
                    <textarea value={context} onChange={(e) => setContext(e.target.value)} className="w-full mt-1 bg-white border border-slate-200 rounded-lg p-3 text-sm h-24 resize-none focus:ring-2 focus:ring-indigo-500 outline-none" />
                </div>
               </div>

              {/* UPLOADS */}
               <div className="grid grid-cols-2 gap-3">
                  <div onClick={() => fileInputRef.current?.click()} className={`cursor-pointer border border-dashed rounded-lg p-3 flex flex-col items-center justify-center transition-all ${image ? 'border-indigo-500 bg-indigo-50' : 'border-slate-300 hover:bg-slate-50'}`}>
                      <ImageIcon className={`w-4 h-4 mb-1 ${image ? 'text-indigo-600' : 'text-slate-400'}`} />
                      <span className="text-[10px] font-bold text-slate-500 uppercase">{image ? 'Image Ready' : 'Upload Visual'}</span>
                      <input type="file" ref={fileInputRef} onChange={(e) => handleFileUpload(e, 'image')} className="hidden" accept="image/*" />
                  </div>
                  
                  <div onClick={() => pdfInputRef.current?.click()} className={`cursor-pointer border border-dashed rounded-lg p-3 flex flex-col items-center justify-center transition-all ${pdf ? 'border-indigo-500 bg-indigo-50' : 'border-slate-300 hover:bg-slate-50'}`}>
                      <FileText className={`w-4 h-4 mb-1 ${pdf ? 'text-indigo-600' : 'text-slate-400'}`} />
                      <span className="text-[10px] font-bold text-slate-500 uppercase">{pdf ? 'PDF Ready' : 'Add Knowledge'}</span>
                      <input type="file" ref={pdfInputRef} onChange={(e) => handleFileUpload(e, 'pdf')} className="hidden" accept="application/pdf" />
                  </div>
              </div>

              {/* COUNT SLIDER */}
              <div>
                <div className="flex justify-between mb-2">
                  <label className="text-[11px] font-bold text-slate-500 uppercase">Sample Size</label>
                  <span className="text-xs font-bold text-indigo-600 bg-indigo-50 px-2 py-0.5 rounded">{count} Agents</span>
                </div>
                <input 
                  type="range" min="1" max="20" value={count}
                  onChange={(e) => setCount(Number(e.target.value))}
                  className="w-full h-1.5 bg-slate-200 rounded-lg appearance-none cursor-pointer accent-indigo-600"
                />
              </div>

              {/* RUN BUTTON */}
              <button
                onClick={startSimulation}
                disabled={status === 'processing'}
                className={`w-full py-3.5 rounded-lg font-bold text-white flex items-center justify-center gap-2 transition-all text-xs uppercase tracking-widest shadow-lg shadow-indigo-200 ${status === 'processing' ? 'bg-slate-400 cursor-not-allowed' : 'bg-slate-900 hover:bg-slate-800 hover:scale-[1.02]'}`}
              >
                {status === 'processing' ? <Activity className="w-4 h-4 animate-spin" /> : <Play className="w-4 h-4 fill-current" />}
                {status === 'processing' ? 'Initializing Swarm...' : 'Start Simulation'}
              </button>
            </div>
          </div>
        </div>

        {/* RIGHT: RESULTS DASHBOARD */}
        <div className="lg:col-span-8 flex flex-col min-h-[600px]">
          
          {/* TOOLBAR */}
          <div className="flex justify-between items-center mb-6">
            <div>
                <h2 className="text-sm font-bold text-slate-400 uppercase tracking-widest flex items-center gap-2">
                <BarChart3 className="w-4 h-4" /> 
                {scenarioType === 'focus_group' ? 'Live Discussion Feed' : 'Consumer Insights Grid'}
                </h2>
            </div>
            
            {/* ACTION BUTTONS (Only show when complete) */}
            {status === 'completed' && (
                <div className="flex items-center gap-2">
                    <button 
                        onClick={generateReport} 
                        disabled={analyzing}
                        className="h-9 px-4 rounded-md text-[10px] font-bold flex items-center gap-2 border border-indigo-200 text-indigo-700 bg-indigo-50 hover:bg-indigo-100 transition-all uppercase tracking-wide"
                    >
                        {analyzing ? <Loader2 className="w-3 h-3 animate-spin" /> : <Sparkles className="w-3 h-3" />} 
                        {analyzing ? "Analyzing..." : "Generate Report"}
                    </button>

                    <button onClick={downloadReport} className="h-9 px-4 rounded-md text-[10px] font-bold flex items-center gap-2 bg-white border border-slate-200 text-slate-600 hover:bg-slate-50 transition-all uppercase tracking-wide">
                        <Download className="w-3 h-3" /> CSV
                    </button>
                </div>
            )}
          </div>

          {/* MAIN CANVAS */}
          <div className="flex-1 bg-white rounded-xl shadow-sm border border-slate-200 overflow-hidden relative">
             {agents.length === 0 ? (
                 <div className="absolute inset-0 flex flex-col items-center justify-center text-slate-300">
                     <BrainCircuit className="w-16 h-16 mb-4 opacity-20" />
                     <p className="text-sm font-medium">Ready to initialize neural swarm.</p>
                     <p className="text-xs">Configure parameters on the left to begin.</p>
                 </div>
             ) : (
                <div className="h-full overflow-y-auto p-6 bg-slate-50/50">
                    {scenarioType === 'focus_group' ? (
                        <FocusGroupChat agents={agents} />
                    ) : (
                        <AgentGrid agents={agents} />
                    )}
                </div>
             )}
          </div>
        </div>
      </div>

      {/* REPORT MODAL */}
      <AnimatePresence>
        {report && (
            <motion.div 
                initial={{ opacity: 0 }} 
                animate={{ opacity: 1 }} 
                exit={{ opacity: 0 }}
                className="fixed inset-0 bg-slate-900/40 backdrop-blur-sm z-50 flex items-center justify-center p-4 md:p-8"
            >
                <motion.div 
                    initial={{ scale: 0.95, y: 10 }} 
                    animate={{ scale: 1, y: 0 }}
                    className="bg-white rounded-xl shadow-2xl w-full max-w-4xl max-h-[90vh] flex flex-col overflow-hidden"
                >
                    <div className="p-5 border-b border-slate-100 flex justify-between items-center bg-slate-50/80">
                        <div className="flex items-center gap-3">
                            <div className="p-2 bg-indigo-100 rounded-lg text-indigo-700">
                                <BookOpen className="w-5 h-5" />
                            </div>
                            <div>
                                <h3 className="text-base font-bold text-slate-800">Strategic Intelligence Report</h3>
                                <p className="text-[10px] text-slate-500 font-bold uppercase tracking-wider">Generated by Analyst Engine</p>
                            </div>
                        </div>
                        <button onClick={() => setReport(null)} className="p-2 hover:bg-slate-200 rounded-full transition-colors">
                            <X className="w-5 h-5 text-slate-500" />
                        </button>
                    </div>
                    
                    <div className="p-8 overflow-y-auto bg-white">
                        <article className="prose prose-sm prose-slate max-w-none prose-headings:font-bold prose-h1:text-xl prose-a:text-indigo-600">
                            <pre className="whitespace-pre-wrap font-sans text-sm text-slate-600 leading-relaxed">
                                {report}
                            </pre>
                        </article>
                    </div>
                    
                    <div className="p-4 bg-slate-50 border-t border-slate-100 flex justify-end">
                         <button 
                            onClick={() => {
                                const blob = new Blob([report], { type: "text/markdown;charset=utf-8;" });
                                const link = document.createElement("a");
                                link.href = URL.createObjectURL(blob);
                                link.download = `ORACULUM_REPORT_${jobId}.md`;
                                link.click();
                            }}
                            className="px-6 py-2.5 rounded-lg bg-slate-900 text-white font-bold text-xs uppercase tracking-wide shadow hover:bg-slate-800 transition-all flex items-center gap-2"
                        >
                            <Download className="w-4 h-4" /> Download Markdown
                        </button>
                    </div>
                </motion.div>
            </motion.div>
        )}
      </AnimatePresence>

    </main>
  );
}
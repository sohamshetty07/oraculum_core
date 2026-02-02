"use client";

import { useState, useEffect, useRef, useCallback } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { 
  Play, Activity, Zap, FileText, Download, 
  Image as ImageIcon, X, Target, Sparkles, BookOpen, 
  BarChart3, Globe, ChevronDown, Loader2,
  BrainCircuit, CheckCircle2, Layers, Search, Briefcase
} from "lucide-react";

import AgentGrid from "../components/AgentGrid";
import FocusGroupChat from "../components/FocusGroupChat";

// --- TYPES ---
interface Agent {
  id?: number | string;
  agent_id?: number | string;
  name?: string;
  role?: string;
  agent_role?: string;
  demographic?: string;
  agent_demographic?: string;
  response?: string;
  thought_process?: string;
  sources?: string; // New field for Phase 6
  sentiment?: string;
  category?: string;
}

// --- COMPONENT: DUAL RANGE SLIDER ---
// A custom implementation to handle Min/Max age on a single track
const DualRangeSlider = ({ min, max, minVal, maxVal, setMinVal, setMaxVal }: any) => {
  const minValRef = useRef(minVal);
  const maxValRef = useRef(maxVal);
  const range = useRef<HTMLDivElement>(null);

  // Convert to percentage
  const getPercent = useCallback(
    (value: number) => Math.round(((value - min) / (max - min)) * 100),
    [min, max]
  );

  // Update range width/position
  useEffect(() => {
    const minPercent = getPercent(minVal);
    const maxPercent = getPercent(maxValRef.current);

    if (range.current) {
      range.current.style.left = `${minPercent}%`;
      range.current.style.width = `${maxPercent - minPercent}%`;
    }
  }, [minVal, getPercent]);

  useEffect(() => {
    const minPercent = getPercent(minValRef.current);
    const maxPercent = getPercent(maxVal);

    if (range.current) {
      range.current.style.width = `${maxPercent - minPercent}%`;
    }
  }, [maxVal, getPercent]);

  return (
    <div className="relative w-full h-8 flex items-center">
      {/* Invisible Inputs */}
      <input
        type="range"
        min={min}
        max={max}
        value={minVal}
        onChange={(event) => {
          const value = Math.min(Number(event.target.value), maxVal - 1);
          setMinVal(value);
          minValRef.current = value;
        }}
        className="thumb thumb--left z-[3] absolute w-full h-0 outline-none pointer-events-none"
        style={{ zIndex: minVal > max - 10 ? "5" : "3" }}
      />
      <input
        type="range"
        min={min}
        max={max}
        value={maxVal}
        onChange={(event) => {
          const value = Math.max(Number(event.target.value), minVal + 1);
          setMaxVal(value);
          maxValRef.current = value;
        }}
        className="thumb thumb--right z-[4] absolute w-full h-0 outline-none pointer-events-none"
      />

      {/* Visual Track */}
      <div className="relative w-full">
        <div className="absolute w-full h-1.5 bg-slate-200 rounded-full z-[1]" />
        <div ref={range} className="absolute h-1.5 bg-indigo-600 rounded-full z-[2]" />
      </div>
      
      {/* Inline Styles for the custom thumbs */}
      <style jsx>{`
        .thumb::-webkit-slider-thumb {
          -webkit-appearance: none;
          -webkit-tap-highlight-color: transparent;
          pointer-events: auto;
          height: 16px;
          width: 16px;
          border-radius: 50%;
          background-color: white;
          border: 2px solid #4f46e5;
          box-shadow: 0 1px 3px rgba(0,0,0,0.3);
          cursor: pointer;
          margin-top: 1px; 
        }
        .thumb::-moz-range-thumb {
          pointer-events: auto;
          height: 16px;
          width: 16px;
          border-radius: 50%;
          background-color: white;
          border: 2px solid #4f46e5;
          box-shadow: 0 1px 3px rgba(0,0,0,0.3);
          cursor: pointer;
        }
      `}</style>
    </div>
  );
};


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
  const [progress, setProgress] = useState(0); 
  
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
    setProgress(5);
    
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
      alert("Backend Connection Error. Ensure Oraculum Core is running on port 8080.");
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

        if (data.progress) setProgress(Math.round(data.progress * 100));

        if (data.results && data.results.length > 0) {
            if (scenarioType === 'focus_group') {
                setAgents(data.results);
            } else {
                setAgents(prev => {
                    const newMap = new Map(prev.map(a => [a.id || a.agent_id, a]));
                    data.results.forEach((r: any) => {
                         const id = r.agent_id || r.id;
                         newMap.set(id, { ...newMap.get(id), ...r, id });
                    });
                    return Array.from(newMap.values()) as Agent[];
                });
            }
        } 
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
    const headers = ["ID", "Category/Round", "Role", "Demographic", "Verdict", "Thought Process", "Sources"];
    
    const rows = agents.map(a => {
        const id = a.agent_id || a.id || "N/A";
        const category = a.category || "General";
        const role = a.agent_role || a.role || a.name || "Participant";
        const demo = a.agent_demographic || a.demographic || "";
        const response = (a.response || "").replace(/"/g, '""');
        const thoughts = (a.thought_process || "").replace(/"/g, '""');
        const sources = (a.sources || "").replace(/"/g, '""');

        return [
            id, `"${category}"`, `"${role}"`, `"${demo}"`, `"${response}"`, `"${thoughts}"`, `"${sources}"`
        ].join(",");
    });

    const csvContent = [headers.join(","), ...rows].join("\n");
    const blob = new Blob([csvContent], { type: "text/csv;charset=utf-8;" });
    const link = document.createElement("a");
    link.href = URL.createObjectURL(blob);
    link.download = `ORACULUM_${scenarioType}_${new Date().toISOString().slice(0,10)}.csv`;
    link.click();
  };

  return (
    <main className="min-h-screen bg-slate-50 font-sans text-slate-900 flex flex-col selection:bg-indigo-100">
      
      {/* 1. TOP NAVIGATION BAR */}
      <header className="bg-white border-b border-slate-200 sticky top-0 z-30 shadow-sm/50 backdrop-blur-sm bg-white/90">
        <div className="max-w-[1800px] mx-auto px-6 h-16 flex items-center justify-between">
            <div className="flex items-center gap-3">
                <div className="bg-slate-900 p-2 rounded-lg text-white shadow-md shadow-indigo-900/10">
                    <BrainCircuit className="w-5 h-5" />
                </div>
                <div>
                    <h1 className="text-lg font-bold tracking-tight text-slate-900 leading-none">Oraculum <span className="text-indigo-600">Core</span></h1>
                    <p className="text-[10px] text-slate-500 font-semibold uppercase tracking-widest mt-0.5">Synthetic Market Intelligence v4.2</p>
                </div>
            </div>

            {/* STATUS & CONTROLS */}
            <div className="flex items-center gap-6">
                {status !== 'idle' && (
                    <div className="flex flex-col items-end min-w-[140px]">
                        <div className="flex items-center gap-2 text-[10px] font-bold text-slate-500 uppercase tracking-wider mb-1.5">
                            {status === 'processing' && <Loader2 className="w-3 h-3 animate-spin text-indigo-600" />}
                            {status === 'processing' ? 'Simulation Active' : 'Analysis Complete'}
                        </div>
                        <div className="w-full h-1.5 bg-slate-100 rounded-full overflow-hidden">
                            <motion.div 
                                className="h-full bg-gradient-to-r from-indigo-500 to-purple-500 rounded-full"
                                initial={{ width: 0 }}
                                animate={{ width: `${progress}%` }}
                                transition={{ duration: 0.5 }}
                            />
                        </div>
                    </div>
                )}
                
                <div className="h-9 px-3 rounded-md bg-slate-50 border border-slate-200 flex items-center gap-2.5 text-[11px] font-bold text-slate-600 uppercase tracking-wide">
                    <div className={`w-2 h-2 rounded-full ${status === 'processing' ? 'bg-emerald-500 animate-pulse shadow-[0_0_8px_rgba(16,185,129,0.5)]' : 'bg-slate-300'}`} />
                    System Ready
                </div>
            </div>
        </div>
      </header>

      {/* 2. MAIN WORKSPACE */}
      <div className="flex-1 max-w-[1800px] mx-auto w-full p-6 grid grid-cols-1 lg:grid-cols-12 gap-8">
        
        {/* LEFT: MISSION CONTROL */}
        <div className="lg:col-span-4 space-y-6">
          
          <div className="bg-white rounded-xl shadow-[0_2px_12px_-4px_rgba(0,0,0,0.05)] border border-slate-200 overflow-hidden">
            <div className="p-4 border-b border-slate-100 bg-slate-50/50 flex items-center gap-2">
                <Zap className="w-4 h-4 text-indigo-600 fill-indigo-600/10" />
                <h2 className="text-xs font-bold text-slate-800 uppercase tracking-widest">Configuration Matrix</h2>
            </div>

            <div className="p-6 space-y-6">
              
              {/* SCENARIO SELECTOR */}
              <div className="space-y-2.5">
                <label className="text-[11px] font-bold text-slate-500 uppercase tracking-wide flex items-center gap-1.5">
                    <Layers className="w-3 h-3" /> Simulation Protocol
                </label>
                <div className="relative group">
                  <select 
                    value={scenarioType}
                    onChange={(e) => setScenarioType(e.target.value)}
                    className="w-full appearance-none bg-white border border-slate-200 text-slate-700 text-sm rounded-lg p-3 pr-8 focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 outline-none transition-all font-medium hover:border-slate-300 shadow-sm"
                  >
                    <option value="product_launch">Product Launch Analysis</option>
                    <option value="focus_group">Multi-Agent Focus Group (Debate)</option>
                    <option value="ab_messaging">Messaging Strategy A/B</option>
                    <option value="creative_test">Creative Asset Evaluation</option>
                    <option value="cx_flow">Customer Journey Simulation</option>
                  </select>
                  <ChevronDown className="absolute right-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-400 pointer-events-none group-hover:text-slate-600 transition-colors" />
                </div>
              </div>

              {/* DEMOGRAPHIC MATRIX */}
              <div className="bg-slate-50/80 rounded-xl p-5 border border-slate-200 space-y-5">
                <div className="flex items-center gap-2 text-[11px] font-bold text-slate-800 uppercase tracking-wide">
                   <Target className="w-3.5 h-3.5 text-indigo-600" /> Target Audience Profile
                </div>
                
                {/* Region & Class */}
                <div className="grid grid-cols-2 gap-4">
                    <div>
                        <span className="text-[10px] font-bold text-slate-400 uppercase tracking-wider block mb-1.5">Region / City</span>
                        <div className="relative">
                            <Globe className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-slate-400" />
                            <input 
                                type="text" value={region} onChange={(e) => setRegion(e.target.value)} 
                                className="w-full pl-8 bg-white border border-slate-200 rounded-md p-2 text-xs font-medium focus:ring-1 focus:ring-indigo-500 focus:border-indigo-500 outline-none transition-all placeholder:text-slate-300"
                                placeholder="e.g. Mumbai"
                            />
                        </div>
                    </div>
                    <div>
                        <span className="text-[10px] font-bold text-slate-400 uppercase tracking-wider block mb-1.5">Socio-Econ Class</span>
                        <div className="relative">
                            <Briefcase className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-slate-400" />
                            <select value={sec} onChange={(e) => setSec(e.target.value)} className="w-full pl-8 bg-white border border-slate-200 rounded-md p-2 text-xs font-medium outline-none focus:ring-1 focus:ring-indigo-500 focus:border-indigo-500 cursor-pointer">
                                <option>SEC A (Premium)</option>
                                <option>SEC B (Middle)</option>
                                <option>SEC C (Mass)</option>
                            </select>
                        </div>
                    </div>
                </div>

                {/* Age Range Slider */}
                <div>
                     <div className="flex justify-between items-center mb-3">
                        <span className="text-[10px] font-bold text-slate-400 uppercase tracking-wider">Age Bracket</span>
                        <span className="text-[10px] font-bold text-indigo-600 bg-indigo-50 px-2 py-0.5 rounded border border-indigo-100">{ageMin} - {ageMax} Years</span>
                     </div>
                     <div className="px-1">
                        <DualRangeSlider 
                            min={18} max={70} 
                            minVal={ageMin} maxVal={ageMax} 
                            setMinVal={setAgeMin} setMaxVal={setAgeMax} 
                        />
                     </div>
                </div>

                {/* Summary */}
                <div className="pt-4 border-t border-slate-200/60">
                    <div className="flex items-start gap-2.5 bg-white p-3 rounded-lg border border-slate-100 shadow-sm">
                        <CheckCircle2 className="w-4 h-4 text-emerald-500 mt-0.5 shrink-0" />
                        <p className="text-xs text-slate-600 font-medium leading-relaxed">{getConstructedAudience()}</p>
                    </div>
                </div>
              </div>

               {/* PRODUCT CONTEXT */}
               <div className="space-y-4">
                 <div className="space-y-1.5">
                    <label className="text-[11px] font-bold text-slate-500 uppercase tracking-wide">Product / Brand Name</label>
                    <input type="text" value={product} onChange={(e) => setProduct(e.target.value)} className="w-full bg-white border border-slate-200 rounded-lg p-3 text-sm font-medium focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 outline-none transition-all shadow-sm" />
                </div>
                <div className="space-y-1.5">
                    <label className="text-[11px] font-bold text-slate-500 uppercase tracking-wide">{getContextLabel()}</label>
                    <textarea value={context} onChange={(e) => setContext(e.target.value)} className="w-full bg-white border border-slate-200 rounded-lg p-3 text-sm h-28 resize-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 outline-none transition-all shadow-sm leading-relaxed" />
                </div>
               </div>

              {/* UPLOADS */}
               <div className="grid grid-cols-2 gap-4">
                  <div onClick={() => fileInputRef.current?.click()} className={`group cursor-pointer border border-dashed rounded-lg p-4 flex flex-col items-center justify-center transition-all duration-200 ${image ? 'border-indigo-500 bg-indigo-50/50' : 'border-slate-300 hover:border-indigo-400 hover:bg-slate-50'}`}>
                      <div className={`p-2 rounded-full mb-2 transition-colors ${image ? 'bg-indigo-100 text-indigo-600' : 'bg-slate-100 text-slate-400 group-hover:text-indigo-500 group-hover:bg-indigo-50'}`}>
                        <ImageIcon className="w-5 h-5" />
                      </div>
                      <span className={`text-[10px] font-bold uppercase tracking-wide ${image ? 'text-indigo-700' : 'text-slate-500 group-hover:text-indigo-600'}`}>{image ? 'Image Loaded' : 'Upload Visual'}</span>
                      <input type="file" ref={fileInputRef} onChange={(e) => handleFileUpload(e, 'image')} className="hidden" accept="image/*" />
                  </div>
                  
                  <div onClick={() => pdfInputRef.current?.click()} className={`group cursor-pointer border border-dashed rounded-lg p-4 flex flex-col items-center justify-center transition-all duration-200 ${pdf ? 'border-indigo-500 bg-indigo-50/50' : 'border-slate-300 hover:border-indigo-400 hover:bg-slate-50'}`}>
                      <div className={`p-2 rounded-full mb-2 transition-colors ${pdf ? 'bg-indigo-100 text-indigo-600' : 'bg-slate-100 text-slate-400 group-hover:text-indigo-500 group-hover:bg-indigo-50'}`}>
                        <FileText className="w-5 h-5" />
                      </div>
                      <span className={`text-[10px] font-bold uppercase tracking-wide ${pdf ? 'text-indigo-700' : 'text-slate-500 group-hover:text-indigo-600'}`}>{pdf ? 'Context Added' : 'Add Knowledge'}</span>
                      <input type="file" ref={pdfInputRef} onChange={(e) => handleFileUpload(e, 'pdf')} className="hidden" accept="application/pdf" />
                  </div>
              </div>

              {/* COUNT SLIDER */}
              <div className="pt-2">
                <div className="flex justify-between mb-3">
                  <label className="text-[11px] font-bold text-slate-500 uppercase tracking-wide">Sample Size</label>
                  <span className="text-xs font-bold text-indigo-600 bg-indigo-50 px-2.5 py-1 rounded border border-indigo-100 shadow-sm">{count} Agents</span>
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
                className={`w-full py-4 rounded-xl font-bold text-white flex items-center justify-center gap-3 transition-all text-xs uppercase tracking-widest shadow-xl shadow-indigo-900/10 ${status === 'processing' ? 'bg-slate-400 cursor-not-allowed' : 'bg-slate-900 hover:bg-indigo-700 hover:-translate-y-0.5 active:translate-y-0'}`}
              >
                {status === 'processing' ? <Activity className="w-4 h-4 animate-spin" /> : <Play className="w-4 h-4 fill-current" />}
                {status === 'processing' ? 'Initializing Neural Swarm...' : 'Initialize Simulation'}
              </button>
            </div>
          </div>
        </div>

        {/* RIGHT: RESULTS DASHBOARD */}
        <div className="lg:col-span-8 flex flex-col min-h-[600px] gap-6">
          
          {/* TOOLBAR */}
          <div className="flex justify-between items-center px-1">
            <div className="flex items-center gap-3">
                <div className="p-2 bg-white border border-slate-200 rounded-lg shadow-sm">
                    <BarChart3 className="w-5 h-5 text-indigo-600" />
                </div>
                <div>
                    <h2 className="text-sm font-bold text-slate-800 uppercase tracking-widest">
                    {scenarioType === 'focus_group' ? 'Live Discussion Feed' : 'Consumer Insights Grid'}
                    </h2>
                    <p className="text-[10px] font-semibold text-slate-400 uppercase tracking-wider mt-0.5">Real-time Inference Stream</p>
                </div>
            </div>
            
            {/* ACTION BUTTONS */}
            {status === 'completed' && (
                <div className="flex items-center gap-3">
                    <button 
                        onClick={generateReport} 
                        disabled={analyzing}
                        className="h-10 px-5 rounded-lg text-[11px] font-bold flex items-center gap-2 border border-indigo-200 text-indigo-700 bg-indigo-50 hover:bg-indigo-100 hover:border-indigo-300 transition-all uppercase tracking-wide shadow-sm"
                    >
                        {analyzing ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Sparkles className="w-3.5 h-3.5" />} 
                        {analyzing ? "Analyzing..." : "Generate Report"}
                    </button>

                    <button onClick={downloadReport} className="h-10 px-5 rounded-lg text-[11px] font-bold flex items-center gap-2 bg-white border border-slate-200 text-slate-600 hover:bg-slate-50 hover:border-slate-300 hover:text-slate-800 transition-all uppercase tracking-wide shadow-sm">
                        <Download className="w-3.5 h-3.5" /> CSV
                    </button>
                </div>
            )}
          </div>

          {/* MAIN CANVAS */}
          <div className="flex-1 bg-white rounded-2xl shadow-sm border border-slate-200 overflow-hidden relative min-h-[600px]">
             {agents.length === 0 ? (
                 <div className="absolute inset-0 flex flex-col items-center justify-center text-slate-300 bg-slate-50/50">
                     <div className="w-20 h-20 bg-white rounded-full flex items-center justify-center mb-6 shadow-sm border border-slate-100">
                        <BrainCircuit className="w-10 h-10 text-slate-300 opacity-80" />
                     </div>
                     <p className="text-sm font-bold text-slate-400 uppercase tracking-widest">Neural Engine Standby</p>
                     <p className="text-xs text-slate-400 mt-2">Configure parameters to begin simulation</p>
                 </div>
             ) : (
                <div className="h-full overflow-y-auto p-6 bg-slate-50/30">
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
                className="fixed inset-0 bg-slate-900/60 backdrop-blur-sm z-50 flex items-center justify-center p-4 md:p-8"
            >
                <motion.div 
                    initial={{ scale: 0.95, y: 10 }} 
                    animate={{ scale: 1, y: 0 }}
                    className="bg-white rounded-2xl shadow-2xl w-full max-w-4xl max-h-[85vh] flex flex-col overflow-hidden border border-white/20"
                >
                    <div className="p-6 border-b border-slate-100 flex justify-between items-center bg-slate-50/80 backdrop-blur-md">
                        <div className="flex items-center gap-4">
                            <div className="p-3 bg-indigo-100 rounded-xl text-indigo-600 shadow-sm border border-indigo-200/50">
                                <BookOpen className="w-6 h-6" />
                            </div>
                            <div>
                                <h3 className="text-lg font-bold text-slate-800 leading-none">Strategic Intelligence Report</h3>
                                <p className="text-[11px] text-slate-500 font-bold uppercase tracking-wider mt-1.5">Generated by Analyst Engine</p>
                            </div>
                        </div>
                        <button onClick={() => setReport(null)} className="p-2 hover:bg-slate-200 rounded-full transition-colors group">
                            <X className="w-5 h-5 text-slate-400 group-hover:text-slate-600" />
                        </button>
                    </div>
                    
                    <div className="p-8 overflow-y-auto bg-white custom-scrollbar">
                        <article className="prose prose-sm prose-slate max-w-none prose-headings:font-bold prose-h1:text-2xl prose-h2:text-lg prose-h2:text-indigo-900 prose-a:text-indigo-600 prose-strong:text-slate-900">
                            <pre className="whitespace-pre-wrap font-sans text-sm text-slate-600 leading-relaxed">
                                {report}
                            </pre>
                        </article>
                    </div>
                    
                    <div className="p-5 bg-slate-50 border-t border-slate-100 flex justify-end">
                         <button 
                            onClick={() => {
                                const blob = new Blob([report], { type: "text/markdown;charset=utf-8;" });
                                const link = document.createElement("a");
                                link.href = URL.createObjectURL(blob);
                                link.download = `ORACULUM_REPORT_${jobId}.md`;
                                link.click();
                            }}
                            className="px-6 py-3 rounded-lg bg-slate-900 text-white font-bold text-xs uppercase tracking-wide shadow-lg shadow-slate-900/10 hover:bg-slate-800 hover:-translate-y-0.5 transition-all flex items-center gap-2"
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
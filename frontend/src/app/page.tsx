"use client";

import { useState, useEffect, useRef } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { 
  Play, Activity, Users, Zap, Layers, FileText, Download, 
  Image as ImageIcon, X, Target, Sparkles, BookOpen, 
  BarChart3, Globe, ShieldCheck, ChevronDown 
} from "lucide-react";

import AgentGrid from "../components/AgentGrid";
import FocusGroupChat from "../components/FocusGroupChat";

export default function Home() {
  // --- STATE ---
  const [product, setProduct] = useState("Cadbury Zero Sugar Silk");
  const [scenarioType, setScenarioType] = useState("product_launch");
  const [context, setContext] = useState("High protein, zero sugar, tastes like real milk chocolate");
  const [count, setCount] = useState(5);
  const [image, setImage] = useState<string | null>(null); 
  const [pdf, setPdf] = useState<string | null>(null);
  
  // DEMOGRAPHIC MATRIX STATE
  const [region, setRegion] = useState("Mumbai");
  const [ageMin, setAgeMin] = useState(18);
  const [ageMax, setAgeMax] = useState(35);
  const [gender, setGender] = useState("Mixed");
  const [sec, setSec] = useState("SEC A (Premium)");

  // Simulation State
  const [status, setStatus] = useState("idle"); 
  const [jobId, setJobId] = useState("");
  const [agents, setAgents] = useState<any[]>([]);
  
  const [analyzing, setAnalyzing] = useState(false);
  const [report, setReport] = useState<string | null>(null);

  const fileInputRef = useRef<HTMLInputElement>(null);
  const pdfInputRef = useRef<HTMLInputElement>(null);

  // --- HANDLERS ---
  
  // Helper to construct the "Target Audience" string from the Matrix
  const getConstructedAudience = () => {
    return `${region} based ${gender === 'Mixed' ? 'Consumers' : gender} aged ${ageMin}-${ageMax}, belonging to ${sec}`;
  };

  const handleImageUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      const reader = new FileReader();
      reader.onloadend = () => setImage((reader.result as string).split(',')[1]);
      reader.readAsDataURL(file);
    }
  };

  const handlePdfUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (file) {
        const reader = new FileReader();
        reader.onloadend = () => setPdf((reader.result as string).split(',')[1]);
        reader.readAsDataURL(file);
      }
  };

  const startSimulation = async () => {
    setStatus("processing");
    setAgents([]); 
    setReport(null);
    
    try {
      const res = await fetch("http://127.0.0.1:8080/api/simulate", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          scenario: scenarioType,
          product_name: product,
          target_audience: getConstructedAudience(), // <--- Using Matrix Data
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
      setStatus("error");
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

  // Polling Logic
  useEffect(() => {
    if (!jobId || status === "completed") return;
    const interval = setInterval(async () => {
      try {
        const res = await fetch(`http://127.0.0.1:8080/api/status/${jobId}`);
        const data = await res.json();

        if (data.results && data.results.length > 0) {
            if (scenarioType === 'focus_group') {
                setAgents(data.results); 
            } else {
                setAgents(prev => {
                    const newMap = new Map(prev.map(a => [a.id || a.agent_id, a]));
                    data.results.forEach((r: any) => newMap.set(r.agent_id, { ...newMap.get(r.agent_id), ...r }));
                    return Array.from(newMap.values());
                });
            }
        } 
        else if (data.agents && agents.length === 0) {
            setAgents(data.agents);
        }

        if (data.status === "completed") {
          setStatus("completed");
          clearInterval(interval);
        }
      } catch (e) {}
    }, 1000); 
    return () => clearInterval(interval);
  }, [jobId, status, count, agents.length, scenarioType]); 

  // --- DOWNLOAD HELPERS ---
  const getSafeFileName = (ext: string) => {
      const safeProduct = product.replace(/[^a-z0-9]/gi, '_').toLowerCase();
      const date = new Date().toISOString().slice(0,10);
      return `${scenarioType}_${safeProduct}_${date}.${ext}`;
  };

  const downloadReport = () => {
    if (agents.length === 0) return;
    const headers = ["ID", "Round", "Role", "Demographic", "Response"];
    const rows = agents.map(a => [
      a.agent_id || a.id,
      `"${a.category || 'N/A'}"`, 
      `"${a.agent_role || a.role || a.agent_name || ''}"`, 
      `"${(a.demographic || a.agent_demographic || '').replace(/"/g, '""')}"`,
      `"${(a.response || '').replace(/"/g, '""').replace(/(\r\n|\n|\r)/gm, " ")}"` 
    ]);
    const csvContent = [headers.join(","), ...rows.map(r => r.join(","))].join("\n");
    const blob = new Blob([csvContent], { type: "text/csv;charset=utf-8;" });
    const link = document.createElement("a");
    link.href = URL.createObjectURL(blob);
    link.download = getSafeFileName("csv");
    link.click();
  };

  const getContextLabel = () => {
     if (scenarioType === "focus_group") return "Discussion Topic";
     if (scenarioType === "creative_test") return "Comparison Copy (Option B)";
     return "Key Benefits / Context";
  };
  
  const positiveCount = agents.filter(a => a.sentiment === 'positive').length;
  const positivePct = agents.length ? Math.round((positiveCount / agents.length) * 100) : 0;

  return (
    <main className="min-h-screen p-6 md:p-12 font-sans relative">
      
      {/* HEADER */}
      <header className="mb-10 flex flex-col md:flex-row justify-between items-start md:items-center gap-6 border-b border-gray-200 pb-6">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-gray-900 flex items-center gap-2">
            <Globe className="w-6 h-6 text-[var(--primary)]" />
            Oraculum <span className="text-gray-400 font-light">| Intelligence Engine</span>
          </h1>
          <p className="text-sm text-gray-500 mt-1">Enterprise Cognitive Simulation Suite v3.2</p>
        </div>
        
        <div className="flex items-center gap-3">
           {status === 'completed' && (
                <>
                    <button 
                        onClick={generateReport} 
                        disabled={analyzing}
                        className="h-10 px-4 rounded-lg text-xs font-semibold flex items-center gap-2 border border-[var(--primary)] text-[var(--primary)] hover:bg-[var(--primary)] hover:text-white transition-all uppercase tracking-wide"
                    >
                        {analyzing ? <Activity className="w-4 h-4 animate-spin" /> : <Sparkles className="w-4 h-4" />} 
                        {analyzing ? "Analyzing..." : "Generate Insights"}
                    </button>

                    <button onClick={downloadReport} className="h-10 px-4 rounded-lg text-xs font-semibold flex items-center gap-2 bg-gray-900 text-white hover:bg-gray-800 transition-all uppercase tracking-wide">
                        <Download className="w-4 h-4" /> Export Data
                    </button>
                </>
            )}
          <div className="h-10 px-4 rounded-lg bg-gray-100 border border-gray-200 flex items-center gap-2 text-xs font-semibold text-gray-600">
            <div className={`w-2 h-2 rounded-full ${status === 'processing' ? 'bg-emerald-500 animate-pulse' : 'bg-gray-400'}`} />
            M4 NEURAL ENGINE
          </div>
        </div>
      </header>

      <div className="grid grid-cols-1 lg:grid-cols-12 gap-8">
        
        {/* LEFT COLUMN: MISSION CONTROL */}
        <div className="lg:col-span-4 space-y-6">
          
          <div className="card-base rounded-xl p-6 relative overflow-hidden">
            <h2 className="text-sm font-bold text-gray-400 uppercase tracking-widest mb-6 flex items-center gap-2">
              <Zap className="w-4 h-4" /> Configuration
            </h2>

            <div className="space-y-6">
              
              {/* SCENARIO */}
              <div>
                <label className="block text-xs font-semibold text-gray-700 mb-2">Simulation Mode</label>
                <div className="relative">
                  <select 
                    value={scenarioType}
                    onChange={(e) => setScenarioType(e.target.value)}
                    className="w-full input-base rounded-lg p-3 pr-10 text-sm appearance-none cursor-pointer"
                  >
                    <option value="product_launch">Product Launch Reaction</option>
                    <option value="focus_group">Multi-Agent Focus Group</option>
                    <option value="ab_messaging">Messaging Strategy A/B</option>
                    <option value="creative_test">Creative Asset Test</option>
                    <option value="cx_flow">Customer Journey Map</option>
                  </select>
                  <ChevronDown className="absolute right-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400 pointer-events-none" />
                </div>
              </div>

              {/* DEMOGRAPHIC MATRIX */}
              <div className="bg-gray-50 rounded-lg p-4 border border-gray-200">
                <label className="block text-xs font-bold text-gray-900 uppercase tracking-wide mb-3 flex items-center gap-2">
                   <Target className="w-3 h-3 text-[var(--primary)]" /> Target Matrix
                </label>
                
                <div className="space-y-3">
                    {/* Region */}
                    <div>
                        <span className="text-[10px] font-semibold text-gray-500 uppercase">Region</span>
                        <input 
                            type="text" 
                            value={region} 
                            onChange={(e) => setRegion(e.target.value)} 
                            className="w-full input-base rounded-md p-2 text-sm mt-1" 
                            placeholder="e.g. Mumbai, Tier 2 Cities"
                        />
                    </div>

                    {/* Age Range */}
                    <div className="flex gap-2">
                        <div className="w-1/2">
                            <span className="text-[10px] font-semibold text-gray-500 uppercase">Min Age</span>
                            <input 
                                type="number" 
                                value={ageMin} 
                                onChange={(e) => setAgeMin(Number(e.target.value))} 
                                className="w-full input-base rounded-md p-2 text-sm mt-1" 
                            />
                        </div>
                        <div className="w-1/2">
                            <span className="text-[10px] font-semibold text-gray-500 uppercase">Max Age</span>
                            <input 
                                type="number" 
                                value={ageMax} 
                                onChange={(e) => setAgeMax(Number(e.target.value))} 
                                className="w-full input-base rounded-md p-2 text-sm mt-1" 
                            />
                        </div>
                    </div>

                    {/* SEC & Gender */}
                    <div className="flex gap-2">
                         <div className="w-1/2">
                            <span className="text-[10px] font-semibold text-gray-500 uppercase">Gender</span>
                            <select 
                                value={gender} 
                                onChange={(e) => setGender(e.target.value)} 
                                className="w-full input-base rounded-md p-2 text-sm mt-1"
                            >
                                <option value="Mixed">Mixed</option>
                                <option value="Male">Male</option>
                                <option value="Female">Female</option>
                            </select>
                        </div>
                        <div className="w-1/2">
                            <span className="text-[10px] font-semibold text-gray-500 uppercase">Class (SEC)</span>
                            <select 
                                value={sec} 
                                onChange={(e) => setSec(e.target.value)} 
                                className="w-full input-base rounded-md p-2 text-sm mt-1"
                            >
                                <option value="SEC A (Premium)">SEC A</option>
                                <option value="SEC B (Middle)">SEC B</option>
                                <option value="SEC C (Mass)">SEC C</option>
                            </select>
                        </div>
                    </div>
                    
                    {/* Live Preview */}
                    <div className="mt-2 pt-2 border-t border-gray-200">
                        <p className="text-[10px] text-gray-400">Constructed Audience:</p>
                        <p className="text-xs font-medium text-[var(--primary)] leading-tight">{getConstructedAudience()}</p>
                    </div>
                </div>
              </div>

               {/* PRODUCT INPUTS */}
               <div className="space-y-3">
                 <div>
                    <label className="block text-xs font-semibold text-gray-700 mb-1">Product / Brand</label>
                    <input type="text" value={product} onChange={(e) => setProduct(e.target.value)} className="w-full input-base rounded-lg p-3 text-sm" />
                </div>
                <div>
                    <label className="block text-xs font-semibold text-gray-700 mb-1">{getContextLabel()}</label>
                    <textarea value={context} onChange={(e) => setContext(e.target.value)} className="w-full input-base rounded-lg p-3 text-sm h-20 resize-none" />
                </div>
               </div>

              {/* UPLOADS */}
               <div className="grid grid-cols-2 gap-3">
                  <div onClick={() => fileInputRef.current?.click()} className={`cursor-pointer border border-dashed rounded-lg p-3 flex flex-col items-center justify-center transition-colors ${image ? 'border-[var(--primary)] bg-indigo-50' : 'border-gray-300 hover:border-gray-400'}`}>
                      <ImageIcon className={`w-5 h-5 mb-1 ${image ? 'text-[var(--primary)]' : 'text-gray-400'}`} />
                      <span className="text-[10px] font-medium text-gray-500">{image ? 'Image Loaded' : 'Add Visual'}</span>
                      <input type="file" ref={fileInputRef} onChange={handleImageUpload} className="hidden" accept="image/*" />
                  </div>
                  
                  <div onClick={() => pdfInputRef.current?.click()} className={`cursor-pointer border border-dashed rounded-lg p-3 flex flex-col items-center justify-center transition-colors ${pdf ? 'border-emerald-500 bg-emerald-50' : 'border-gray-300 hover:border-gray-400'}`}>
                      <FileText className={`w-5 h-5 mb-1 ${pdf ? 'text-emerald-500' : 'text-gray-400'}`} />
                      <span className="text-[10px] font-medium text-gray-500">{pdf ? 'PDF Loaded' : 'Add Knowledge'}</span>
                      <input type="file" ref={pdfInputRef} onChange={handlePdfUpload} className="hidden" accept="application/pdf" />
                  </div>
              </div>

              {/* COUNT SLIDER */}
              <div>
                <div className="flex justify-between mb-2">
                  <label className="text-xs font-semibold text-gray-700">Sample Size</label>
                  <span className="text-xs font-bold text-[var(--primary)]">{count} Agents</span>
                </div>
                <input 
                  type="range" 
                  min="1" max="25" 
                  value={count}
                  onChange={(e) => setCount(Number(e.target.value))}
                  className="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-[var(--primary)]"
                />
              </div>

              {/* ACTION BUTTON */}
              <button
                onClick={startSimulation}
                disabled={status === 'processing'}
                className={`w-full py-3 rounded-lg font-bold text-white flex items-center justify-center gap-2 transition-all text-sm uppercase tracking-wide ${status === 'processing' ? 'bg-gray-400' : 'bg-[var(--primary)] hover:bg-[var(--primary-hover)] shadow-md'}`}
              >
                {status === 'processing' ? <Activity className="w-4 h-4 animate-spin" /> : <Play className="w-4 h-4 fill-current" />}
                {status === 'processing' ? 'Initializing Swarm...' : 'Run Simulation'}
              </button>
            </div>
          </div>

          {/* METRICS */}
          <div className="grid grid-cols-2 gap-4">
            <div className="card-base rounded-xl p-5 flex flex-col items-center justify-center">
              <div className="text-3xl font-bold text-gray-900">{agents.length}</div>
              <div className="text-[10px] font-bold uppercase text-gray-400 mt-1">Responses</div>
            </div>
            <div className="card-base rounded-xl p-5 flex flex-col items-center justify-center">
              <div className="text-3xl font-bold text-emerald-600">
                {positivePct}<span className="text-lg">%</span>
              </div>
              <div className="text-[10px] font-bold uppercase text-gray-400 mt-1">Positive Sentiment</div>
            </div>
          </div>

        </div>

        {/* RIGHT COLUMN: RESULTS */}
        <div className="lg:col-span-8">
          <div className="flex justify-between items-center mb-6">
            <h2 className="text-sm font-bold text-gray-400 uppercase tracking-widest flex items-center gap-2">
              <BarChart3 className="w-4 h-4" /> 
              {scenarioType === 'focus_group' ? 'Live Discussion Feed' : 'Agent Responses'}
            </h2>
            {status === 'processing' && <span className="text-xs font-mono font-bold text-[var(--primary)] animate-pulse">LIVE PROCESSING...</span>}
          </div>

          {scenarioType === 'focus_group' ? (
              <FocusGroupChat agents={agents} />
          ) : (
              <AgentGrid agents={agents} />
          )}

        </div>
      </div>

      {/* REPORT MODAL */}
      <AnimatePresence>
        {report && (
            <motion.div 
                initial={{ opacity: 0 }} 
                animate={{ opacity: 1 }} 
                exit={{ opacity: 0 }}
                className="fixed inset-0 bg-gray-900/50 backdrop-blur-sm z-50 flex items-center justify-center p-4 md:p-12"
            >
                <motion.div 
                    initial={{ scale: 0.95, y: 10 }} 
                    animate={{ scale: 1, y: 0 }}
                    className="bg-white rounded-xl shadow-2xl w-full max-w-4xl max-h-full overflow-hidden flex flex-col border border-gray-200"
                >
                    <div className="p-5 border-b border-gray-100 flex justify-between items-center bg-gray-50">
                        <div className="flex items-center gap-3">
                            <div className="p-2 bg-indigo-50 rounded-lg text-[var(--primary)]">
                                <ShieldCheck className="w-5 h-5" />
                            </div>
                            <div>
                                <h3 className="text-lg font-bold text-gray-900">Intelligence Report</h3>
                                <p className="text-xs text-gray-500 font-medium uppercase tracking-wider">Oraculum Analyst Engine</p>
                            </div>
                        </div>
                        <button onClick={() => setReport(null)} className="p-2 hover:bg-gray-200 rounded-full transition-colors">
                            <X className="w-5 h-5 text-gray-500" />
                        </button>
                    </div>
                    
                    <div className="p-8 overflow-y-auto leading-relaxed text-gray-700 space-y-4 bg-white">
                        <div className="prose prose-sm prose-slate max-w-none">
                            <pre className="whitespace-pre-wrap font-sans text-sm">
                                {report}
                            </pre>
                        </div>
                    </div>
                    
                    <div className="p-4 bg-gray-50 border-t border-gray-100 flex justify-end">
                         <button 
                            onClick={() => {
                                const blob = new Blob([report], { type: "text/markdown;charset=utf-8;" });
                                const link = document.createElement("a");
                                link.href = URL.createObjectURL(blob);
                                link.download = getSafeFileName("md");
                                link.click();
                            }}
                            className="px-6 py-2.5 rounded-lg bg-gray-900 text-white font-bold text-xs uppercase tracking-wide shadow hover:bg-gray-800 transition-all flex items-center gap-2"
                        >
                            <Download className="w-4 h-4" /> Download Report
                        </button>
                    </div>
                </motion.div>
            </motion.div>
        )}
      </AnimatePresence>

    </main>
  );
}
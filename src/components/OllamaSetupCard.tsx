import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface OllamaModelConfig {
  preset?: string;
  orchestrator_model?: string;
  system_model?: string;
  code_model?: string;
  reasoning_model?: string;
  general_model?: string;
  vision_model?: string;
  default_model?: string;
}

export interface OllamaStatusData {
  config?: OllamaModelConfig;
  connected: boolean;
  installed_models: string[];
  config_file_path?: string;
  loading: boolean;
}

interface OllamaSetupCardProps {
  ollamaInfo: OllamaStatusData;
  onRefresh: () => Promise<void>;
  compact?: boolean;
}

interface PresetOption {
  id: string;
  name: string;
  badge: string;
  command: string;
  altCommand?: string;
  description: string;
  tag: string;
}

const PRESETS: PresetOption[] = [
  {
    id: 'basico',
    name: 'Básico (CPU / Ligero)',
    badge: 'CPU • 4-8 GB RAM',
    command: 'ollama run llama3.2:1b',
    description: 'Fluido y ágil en cualquier portátil o equipos sin GPU dedicada.',
    tag: 'Ultra-Ligero',
  },
  {
    id: 'equilibrado',
    name: 'Equilibrado (Recomendado)',
    badge: 'GPU Media • 16 GB RAM',
    command: 'ollama run llama3.2:3b',
    description: 'Excelente tiempo de respuesta diario para RTX 2060/3060/4060 o AMD Radeon.',
    tag: 'Recomendado',
  },
  {
    id: 'alto',
    name: 'Alto Rendimiento (RTX 40/50 / Blackwell)',
    badge: '12-32 GB VRAM • Serie 40/50',
    command: 'ollama run hermes3:8b',
    altCommand: 'ollama run qwen2.5:14b',
    description: 'Razonamiento profundo y ejecución de herramientas para hardware de alta gama.',
    tag: 'Potente',
  },
  {
    id: 'personalizado',
    name: 'Perfil Personalizado',
    badge: 'Cualquier modelo',
    command: 'ollama run deepseek-r1:14b',
    description: 'Introduce cualquier modelo del catálogo oficial de Ollama.',
    tag: 'Personalizado',
  },
];

export const OllamaSetupCard: React.FC<OllamaSetupCardProps> = ({
  ollamaInfo,
  onRefresh,
  compact = false,
}) => {
  const [selectedPresetId, setSelectedPresetId] = useState<string>('equilibrado');
  const [customCommand, setCustomCommand] = useState<string>('ollama run deepseek-r1:14b');
  const [copiedCommand, setCopiedCommand] = useState<string | null>(null);
  const [runningCommand, setRunningCommand] = useState<string | null>(null);
  const [isRefreshing, setIsRefreshing] = useState<boolean>(false);

  const selectedPreset = PRESETS.find((p) => p.id === selectedPresetId) || PRESETS[1];
  const effectiveCommand =
    selectedPresetId === 'personalizado' ? customCommand.trim() : selectedPreset.command;

  const handleCopy = async (cmd: string, e?: React.MouseEvent) => {
    if (e) e.stopPropagation();
    const clean = cmd.trim();
    if (!clean) return;
    try {
      await navigator.clipboard.writeText(clean);
      setCopiedCommand(clean);
      setTimeout(() => setCopiedCommand(null), 2500);
    } catch (err) {
      console.error('[OllamaSetupCard] Error copying to clipboard:', err);
    }
  };

  const handleRunInTerminal = async (cmd: string, e?: React.MouseEvent) => {
    if (e) e.stopPropagation();
    const clean = cmd.trim();
    if (!clean) return;
    setRunningCommand(clean);
    try {
      await invoke('run_terminal_command', { command: clean });
      setTimeout(() => setRunningCommand(null), 3500);
    } catch (err) {
      console.error('[OllamaSetupCard] Error running in terminal:', err);
      setRunningCommand(null);
    }
  };

  const handleRefreshClick = async () => {
    setIsRefreshing(true);
    try {
      await onRefresh();
    } finally {
      setTimeout(() => setIsRefreshing(false), 500);
    }
  };

  return (
    <div
      className={`w-full rounded-2xl neu-extruded border border-red-950/40 flex flex-col ${
        compact ? 'p-3 gap-2.5' : 'p-4 gap-3.5'
      } bg-[#0A0A0D] shadow-[4px_4px_12px_#030305,-2px_-2px_8px_#14141A]`}
    >
      {/* Cabecera del Panel */}
      <div className="flex items-start justify-between gap-3">
        <div className="flex items-center gap-2.5">
          <div className="w-8 h-8 rounded-xl neu-inset flex items-center justify-center text-[#EF4444] shrink-0">
            <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"
              />
            </svg>
          </div>
          <div>
            <div className="text-xs font-semibold text-white tracking-wide flex items-center gap-2">
              <span>
                {!ollamaInfo.connected
                  ? 'Ollama Local No Detectado'
                  : 'Sin Modelos de IA Local Detectados'}
              </span>
            </div>
            <div className="text-[10px] text-white/50 leading-relaxed">
              {!ollamaInfo.connected
                ? 'El servicio Ollama no está activo en el puerto 11434. Inícialo o descarga un modelo:'
                : 'Ollama está en ejecución pero no hay modelos descargados en tu sistema local.'}
            </div>
          </div>
        </div>

        <div className="flex items-center gap-1.5 shrink-0">
          <button
            type="button"
            onClick={handleRefreshClick}
            disabled={isRefreshing || ollamaInfo.loading}
            className="neu-extruded px-2 py-1 rounded-lg text-[10px] font-mono text-white/70 hover:text-white flex items-center gap-1 cursor-pointer disabled:opacity-40"
            title="Comprobar de nuevo"
          >
            <svg
              className={`w-3 h-3 ${isRefreshing || ollamaInfo.loading ? 'animate-spin' : ''}`}
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
              />
            </svg>
            <span className="hidden sm:inline">Comprobar</span>
          </button>
          <span className="text-[10px] font-mono neu-inset px-2 py-0.5 rounded-md flex items-center gap-1.5 text-white/80">
            <span
              className={`w-1.5 h-1.5 rounded-full ${
                !ollamaInfo.connected
                  ? 'bg-zinc-600'
                  : 'bg-amber-400 shadow-[0_0_6px_#f59e0b] animate-pulse'
              }`}
            />
            {!ollamaInfo.connected ? 'Offline' : '0 Modelos'}
          </span>
        </div>
      </div>

      {/* Si Ollama está offline, ayuda rápida de instalación */}
      {!ollamaInfo.connected && (
        <div className="p-2.5 rounded-xl bg-red-950/20 border border-red-900/30 flex items-center justify-between gap-2 text-[10px]">
          <div className="text-white/80">
            <span className="text-[#EF4444] font-semibold">Instalación rápida en Windows: </span>
            <code className="text-white font-mono bg-black/40 px-1.5 py-0.5 rounded border border-white/10">
              winget install Ollama.Ollama
            </code>
          </div>
          <button
            type="button"
            onClick={(e) => handleCopy('winget install Ollama.Ollama', e)}
            className="neu-extruded px-2 py-0.5 rounded text-[9px] font-mono text-white/70 hover:text-white shrink-0 cursor-pointer"
          >
            {copiedCommand === 'winget install Ollama.Ollama' ? '✓ Copiado' : 'Copiar winget'}
          </button>
        </div>
      )}

      {/* Lista de Perfiles de Modelos (Bento Cards) */}
      <div className={`grid ${compact ? 'grid-cols-1' : 'grid-cols-1 sm:grid-cols-2'} gap-2`}>
        {PRESETS.map((preset) => {
          const isSelected = selectedPresetId === preset.id;
          return (
            <div
              key={preset.id}
              onClick={() => setSelectedPresetId(preset.id)}
              className={`p-2.5 rounded-xl border transition-all cursor-pointer flex flex-col justify-between gap-1.5 ${
                isSelected
                  ? 'neu-relief bg-red-950/20 border-[#EF4444]/60 shadow-[inset_1px_1px_4px_#020203]'
                  : 'neu-extruded bg-[#08080B] border-white/5 hover:border-white/15'
              }`}
            >
              <div className="flex items-center justify-between w-full">
                <div className="flex items-center gap-1.5">
                  <span
                    className={`w-2 h-2 rounded-full ${
                      isSelected ? 'neu-led-red' : 'bg-white/20'
                    }`}
                  />
                  <span className="text-xs font-semibold text-white tracking-wide">
                    {preset.name}
                  </span>
                </div>
                <span className="text-[9px] font-mono text-white/50 neu-inset px-1.5 py-0.5 rounded">
                  {preset.badge}
                </span>
              </div>

              <div className="text-[10px] text-white/50 leading-relaxed">
                {preset.description}
              </div>

              {preset.id !== 'personalizado' ? (
                <div className="mt-1 flex items-center justify-between gap-1 pt-1.5 border-t border-white/5">
                  <code className="text-[10px] font-mono text-white/90 truncate bg-black/40 px-1.5 py-0.5 rounded border border-white/5">
                    {preset.command}
                  </code>
                  <div className="flex items-center gap-1 shrink-0">
                    <button
                      type="button"
                      onClick={(e) => handleCopy(preset.command, e)}
                      className="p-1 rounded hover:bg-white/10 text-white/60 hover:text-white transition-colors cursor-pointer text-[10px]"
                      title="Copiar comando"
                    >
                      {copiedCommand === preset.command ? (
                        <span className="text-green-400 font-mono text-[9px]">✓</span>
                      ) : (
                        <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            strokeWidth={2}
                            d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
                          />
                        </svg>
                      )}
                    </button>
                    <button
                      type="button"
                      onClick={(e) => handleRunInTerminal(preset.command, e)}
                      className="px-1.5 py-0.5 rounded bg-white/5 hover:bg-white/15 text-white/80 hover:text-white text-[9px] font-mono cursor-pointer"
                      title="Ejecutar directamente en terminal"
                    >
                      ▶ Run
                    </button>
                  </div>
                </div>
              ) : (
                <div className="mt-1 pt-1.5 border-t border-white/5">
                  <input
                    type="text"
                    value={customCommand}
                    onClick={(e) => e.stopPropagation()}
                    onChange={(e) => setCustomCommand(e.target.value)}
                    placeholder="ollama run <modelo>"
                    className="w-full neu-inset bg-[#050507] border border-red-950/40 rounded-lg px-2 py-1 text-[10px] font-mono text-white placeholder-white/30 focus:outline-none focus:border-[#EF4444]"
                  />
                </div>
              )}
            </div>
          );
        })}
      </div>

      {/* Barra de Acciones del Comando Seleccionado */}
      <div className="p-2.5 rounded-xl neu-inset bg-[#060608] border border-red-950/40 flex flex-col sm:flex-row items-center justify-between gap-2">
        <div className="flex items-center gap-2 w-full sm:w-auto truncate">
          <span className="text-[10px] font-mono text-[#EF4444] font-semibold shrink-0">
            Comando:
          </span>
          <code className="text-[11px] font-mono text-white truncate">{effectiveCommand}</code>
        </div>

        <div className="flex items-center gap-2 w-full sm:w-auto justify-end shrink-0">
          <button
            type="button"
            onClick={(e) => handleCopy(effectiveCommand, e)}
            className="flex-1 sm:flex-none neu-extruded bg-[#0e0e13] hover:bg-[#15151c] text-white/80 hover:text-white border border-white/10 px-3 py-1.5 rounded-xl text-xs font-medium flex items-center justify-center gap-1.5 cursor-pointer transition-all active:scale-95"
          >
            <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
              />
            </svg>
            <span>{copiedCommand === effectiveCommand ? '¡Copiado!' : 'Copiar'}</span>
          </button>

          <button
            type="button"
            onClick={(e) => handleRunInTerminal(effectiveCommand, e)}
            disabled={!effectiveCommand || runningCommand === effectiveCommand}
            className="flex-1 sm:flex-none neu-extruded bg-gradient-to-r from-[#B82A2A] to-[#EF4444] hover:from-[#DC2626] hover:to-[#F87171] text-white px-3.5 py-1.5 rounded-xl text-xs font-semibold flex items-center justify-center gap-1.5 shadow-[2px_2px_6px_#040406,-1px_-1px_4px_#1c1c24] cursor-pointer transition-all active:scale-95 disabled:opacity-40"
          >
            <span>⚡</span>
            <span>
              {runningCommand === effectiveCommand ? 'Ejecutando...' : 'Ejecutar en Terminal'}
            </span>
          </button>
        </div>
      </div>
    </div>
  );
};

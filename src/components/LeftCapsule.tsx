import React from 'react';
import { motion } from 'framer-motion';
import { 
  Terminal, 
  Globe, 
  Folder, 
  Code 
} from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { useVirtualDesktops, VirtualDesktopItem } from '../hooks/useVirtualDesktops';

interface LeftCapsuleProps {
  innerRef?: React.Ref<HTMLDivElement>;
  onOpenSearch?: () => void;
}

export const LeftCapsule: React.FC<LeftCapsuleProps> = ({ 
  innerRef,
  onOpenSearch 
}) => {
  const { state, switchDesktop } = useVirtualDesktops();
  const desktops = state.desktops;

  const handleLaunch = (cmd: string) => {
    invoke('launch_app', { execCommand: cmd }).catch((err) => {
      console.warn(`[LeftCapsule] Failed to launch ${cmd}:`, err);
    });
  };

  const appShortcuts = [
    { id: 'term', name: 'Terminal (PowerShell / WT)', icon: Terminal, cmd: 'powershell' },
    { id: 'web', name: 'Navegador Web', icon: Globe, cmd: 'brave' },
    { id: 'files', name: 'Explorador de Archivos', icon: Folder, cmd: 'explorer' },
    { id: 'code', name: 'VS Code', icon: Code, cmd: 'code' },
  ];

  // Desktops fallback seguro en caso de que el sistema tarde en reportar
  const safeDesktops: VirtualDesktopItem[] = desktops && desktops.length > 0 
    ? desktops 
    : [
        { id: '0', index: 0, name: '01', is_current: true },
        { id: '1', index: 1, name: '02', is_current: false },
        { id: '2', index: 2, name: '03', is_current: false },
        { id: '3', index: 3, name: '04', is_current: false },
      ];

  return (
    <motion.div
      ref={innerRef}
      id="left-capsule"
      data-input-region="true"
      animate={{ opacity: [1, 0.999, 1] }}
      transition={{ duration: 2, repeat: Infinity, ease: 'linear' }}
      className="flex items-center h-[34px] pointer-events-auto select-none m-0 p-0"
      style={{ willChange: 'opacity, transform', transform: 'translateZ(0)' }}
    >
      {/* 1. Contenedor Físico Principal: Isla Flotante con bordes redondeados y micro-relieve afilado */}
      <div
        className="flex items-center h-[34px] text-white select-none px-3.5 neu-chassis rounded-2xl shadow-[0_8px_24px_rgba(0,0,0,0.85)] border border-red-950/40"
      >
        <div className="flex items-center gap-2 sm:gap-2.5 h-full">
          {/* 1. Botón Enigma: Micro-relieve limpio con logo Enigma Core y texto blanco */}
          <motion.button
            whileHover={{ scale: 1.05 }}
            whileTap={{ scale: 0.94 }}
            onClick={onOpenSearch}
            className="h-7 flex items-center gap-1.5 px-2.5 rounded-xl whitespace-nowrap bg-[#111116] border border-white/5 hover:border-[#EF4444]/40 shadow-[1px_1px_3px_#000,inset_0_1px_0_rgba(255,255,255,0.06)] text-white cursor-pointer active:scale-95 transition-all"
            title="Enigma Core / Búsqueda y Lanzador (Alt + Espacio)"
          >
            <motion.div
              animate={{ 
                scale: [1, 1.08, 1],
                filter: [
                  'drop-shadow(0 0 3px rgba(239,68,68,0.7))',
                  'drop-shadow(0 0 7px rgba(239,68,68,0.95))',
                  'drop-shadow(0 0 3px rgba(239,68,68,0.7))'
                ]
              }}
              transition={{ duration: 3, repeat: Infinity, ease: 'easeInOut' }}
              className="flex items-center justify-center shrink-0"
            >
              <svg
                className="w-4 h-4"
                viewBox="0 0 24 24"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
              >
                <path
                  d="M12 2L21 7.2V16.8L12 22L3 16.8V7.2L12 2Z"
                  stroke="url(#enigmaRedGrad)"
                  strokeWidth="2"
                  fill="#140608"
                />
                <path
                  d="M9 8.5H15M9 12H13.5M9 15.5H15M9 8.5V15.5"
                  stroke="#FF4D4D"
                  strokeWidth="2"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                />
                <circle cx="15" cy="8.5" r="1" fill="#FFF" />
                <circle cx="13.5" cy="12" r="1" fill="#FFF" />
                <circle cx="15" cy="15.5" r="1" fill="#FFF" />
                <defs>
                  <linearGradient id="enigmaRedGrad" x1="3" y1="2" x2="21" y2="22" gradientUnits="userSpaceOnUse">
                    <stop stopColor="#EF4444" />
                    <stop offset="0.5" stopColor="#DC2626" />
                    <stop offset="1" stopColor="#7F1D1D" />
                  </linearGradient>
                </defs>
              </svg>
            </motion.div>

            <span className="font-mono text-xs font-bold tracking-widest text-white">
              ENIGMA
            </span>
          </motion.button>

          {/* Separador vertical tallado */}
          <div className="w-[1.5px] h-3.5 bg-red-950/80 shadow-[1px_0_0_rgba(255,255,255,0.05),0_0_6px_rgba(239,68,68,0.4)] rounded-full" />

          {/* 2. Indicador de Escritorios Virtuales: Píldora uniforme h-7 con micro-cavidad interior */}
          <div className="h-7 flex items-center gap-1 px-1.5 rounded-full bg-[#08080C] border border-white/5 shadow-[inset_1px_1px_3px_#000,0_1px_0_rgba(255,255,255,0.04)] whitespace-nowrap">
            {safeDesktops.map((ws: VirtualDesktopItem) => {
              const isActive = ws.is_current;
              const label = ws.index !== undefined ? `0${ws.index + 1}` : ws.name;
              return (
                <motion.button
                  key={ws.id || ws.index}
                  onClick={(e) => {
                    e.stopPropagation();
                    switchDesktop(ws.id);
                  }}
                  whileHover={{ scale: 1.06 }}
                  whileTap={{ scale: 0.94 }}
                  className={`relative px-2 py-0.5 rounded-full text-xs font-mono font-bold transition-all cursor-pointer ${
                    isActive
                      ? 'bg-[#181822] text-white border border-[#EF4444]/60 shadow-[0_0_8px_rgba(239,68,68,0.4)]'
                      : 'text-white/60 hover:text-white px-1.5'
                  }`}
                  title={`Cambiar a Escritorio Virtual ${label}`}
                >
                  {label}
                </motion.button>
              );
            })}
          </div>

          {/* Separador vertical tallado (responsivo: visible en pantallas >= 1100px) */}
          <div className="hidden xl:block w-[1.5px] h-3.5 bg-red-950/80 shadow-[1px_0_0_rgba(255,255,255,0.05),0_0_6px_rgba(239,68,68,0.4)] rounded-full" />

          {/* 3. Botones de acceso rápido a aplicaciones (responsivo: visible en pantallas amplias) */}
          <div className="hidden xl:flex items-center gap-1.5">
            {appShortcuts.map((app) => {
              const Icon = app.icon;
              return (
                <motion.button
                  key={app.id}
                  onClick={() => handleLaunch(app.cmd)}
                  whileHover={{ scale: 1.12 }}
                  whileTap={{ scale: 0.92 }}
                  className="w-7 h-7 rounded-full bg-[#111116] border border-white/5 hover:border-[#EF4444]/30 hover:bg-[#16161E] shadow-[1px_1px_3px_#000,inset_0_1px_0_rgba(255,255,255,0.06)] flex items-center justify-center text-[#EF4444] hover:text-white active:scale-95 transition-all cursor-pointer"
                  title={app.name}
                >
                  <Icon className="w-3.5 h-3.5 text-[#EF4444]" />
                </motion.button>
              );
            })}
          </div>
        </div>
      </div>
    </motion.div>
  );
};

export default LeftCapsule;

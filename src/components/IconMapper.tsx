import React from 'react';
import { convertFileSrc } from '@tauri-apps/api/core';
import { 
  SiSpotify, 
  SiDiscord, 
  SiOpera, 
  SiFirefox, 
  SiGooglechrome, 
  SiBrave, 
  SiSteam, 
  SiObsidian, 
  SiTelegram, 
  SiKde 
} from 'react-icons/si';
import { 
  VscVscode, 
  VscTerminalBash, 
  VscFolder,
  VscSettingsGear,
  VscPulse,
  VscTerminalPowershell,
  VscTerminalCmd
} from 'react-icons/vsc';
import {
  BsBrowserEdge,
  BsWindows,
  BsFileText,
  BsCalculator,
  BsPaintBucket
} from 'react-icons/bs';

export interface AppBrandInfo {
  icon: React.ComponentType<{ className?: string; style?: React.CSSProperties }>;
  brandColor: string;
  shadowColor: string;
}

const BRAND_REGISTRY: Record<string, AppBrandInfo> = {
  edge: {
    icon: BsBrowserEdge,
    brandColor: '#0078D7',
    shadowColor: 'rgba(0, 120, 215, 0.55)',
  },
  msedge: {
    icon: BsBrowserEdge,
    brandColor: '#0078D7',
    shadowColor: 'rgba(0, 120, 215, 0.55)',
  },
  'microsoft-edge': {
    icon: BsBrowserEdge,
    brandColor: '#0078D7',
    shadowColor: 'rgba(0, 120, 215, 0.55)',
  },
  browser: {
    icon: BsBrowserEdge,
    brandColor: '#0078D7',
    shadowColor: 'rgba(0, 120, 215, 0.55)',
  },
  navegador: {
    icon: BsBrowserEdge,
    brandColor: '#0078D7',
    shadowColor: 'rgba(0, 120, 215, 0.55)',
  },
  taskmgr: {
    icon: VscPulse,
    brandColor: '#10B981',
    shadowColor: 'rgba(16, 185, 129, 0.5)',
  },
  taskmanager: {
    icon: VscPulse,
    brandColor: '#10B981',
    shadowColor: 'rgba(16, 185, 129, 0.5)',
  },
  'task manager': {
    icon: VscPulse,
    brandColor: '#10B981',
    shadowColor: 'rgba(16, 185, 129, 0.5)',
  },
  rendimiento: {
    icon: VscPulse,
    brandColor: '#10B981',
    shadowColor: 'rgba(16, 185, 129, 0.5)',
  },
  tareas: {
    icon: VscPulse,
    brandColor: '#10B981',
    shadowColor: 'rgba(16, 185, 129, 0.5)',
  },
  powershell: {
    icon: VscTerminalPowershell,
    brandColor: '#2670E8',
    shadowColor: 'rgba(38, 112, 232, 0.5)',
  },
  pwsh: {
    icon: VscTerminalPowershell,
    brandColor: '#2670E8',
    shadowColor: 'rgba(38, 112, 232, 0.5)',
  },
  cmd: {
    icon: VscTerminalCmd,
    brandColor: '#D1D5DB',
    shadowColor: 'rgba(209, 213, 219, 0.4)',
  },
  commandprompt: {
    icon: VscTerminalCmd,
    brandColor: '#D1D5DB',
    shadowColor: 'rgba(209, 213, 219, 0.4)',
  },
  explorer: {
    icon: VscFolder,
    brandColor: '#0078D4',
    shadowColor: 'rgba(0, 120, 212, 0.5)',
  },
  windows: {
    icon: BsWindows,
    brandColor: '#0078D4',
    shadowColor: 'rgba(0, 120, 212, 0.5)',
  },
  spotify: {
    icon: SiSpotify,
    brandColor: '#1ED760',
    shadowColor: 'rgba(30, 215, 96, 0.45)',
  },
  discord: {
    icon: SiDiscord,
    brandColor: '#5865F2',
    shadowColor: 'rgba(88, 101, 242, 0.5)',
  },
  opera: {
    icon: SiOpera,
    brandColor: '#FF1B2D',
    shadowColor: 'rgba(255, 27, 45, 0.5)',
  },
  operagx: {
    icon: SiOpera,
    brandColor: '#FA1E4E',
    shadowColor: 'rgba(250, 30, 78, 0.5)',
  },
  'antigravity-ide': {
    icon: VscVscode,
    brandColor: '#007ACC',
    shadowColor: 'rgba(0, 122, 204, 0.5)',
  },
  antigravity: {
    icon: VscVscode,
    brandColor: '#007ACC',
    shadowColor: 'rgba(0, 122, 204, 0.5)',
  },
  vscode: {
    icon: VscVscode,
    brandColor: '#007ACC',
    shadowColor: 'rgba(0, 122, 204, 0.5)',
  },
  code: {
    icon: VscVscode,
    brandColor: '#007ACC',
    shadowColor: 'rgba(0, 122, 204, 0.5)',
  },
  ide: {
    icon: VscVscode,
    brandColor: '#007ACC',
    shadowColor: 'rgba(0, 122, 204, 0.5)',
  },
  terminal: {
    icon: VscTerminalBash,
    brandColor: '#10B981',
    shadowColor: 'rgba(16, 185, 129, 0.5)',
  },
  kitty: {
    icon: VscTerminalBash,
    brandColor: '#10B981',
    shadowColor: 'rgba(16, 185, 129, 0.5)',
  },
  konsole: {
    icon: VscTerminalBash,
    brandColor: '#10B981',
    shadowColor: 'rgba(16, 185, 129, 0.5)',
  },
  bash: {
    icon: VscTerminalBash,
    brandColor: '#10B981',
    shadowColor: 'rgba(16, 185, 129, 0.5)',
  },
  dolphin: {
    icon: VscFolder,
    brandColor: '#3B82F6',
    shadowColor: 'rgba(59, 130, 246, 0.5)',
  },
  files: {
    icon: VscFolder,
    brandColor: '#3B82F6',
    shadowColor: 'rgba(59, 130, 246, 0.5)',
  },
  archivos: {
    icon: VscFolder,
    brandColor: '#3B82F6',
    shadowColor: 'rgba(59, 130, 246, 0.5)',
  },
  firefox: {
    icon: SiFirefox,
    brandColor: '#FF7139',
    shadowColor: 'rgba(255, 113, 57, 0.55)',
  },
  'google-chrome': {
    icon: SiGooglechrome,
    brandColor: '#4285F4',
    shadowColor: 'rgba(66, 133, 244, 0.55)',
  },
  chrome: {
    icon: SiGooglechrome,
    brandColor: '#4285F4',
    shadowColor: 'rgba(66, 133, 244, 0.55)',
  },
  brave: {
    icon: SiBrave,
    brandColor: '#FB542B',
    shadowColor: 'rgba(251, 84, 43, 0.55)',
  },
  steam: {
    icon: SiSteam,
    brandColor: '#1A9FFF',
    shadowColor: 'rgba(26, 159, 255, 0.55)',
  },
  telegram: {
    icon: SiTelegram,
    brandColor: '#24A1DE',
    shadowColor: 'rgba(36, 161, 222, 0.55)',
  },
  obsidian: {
    icon: SiObsidian,
    brandColor: '#A78BFA',
    shadowColor: 'rgba(167, 139, 250, 0.55)',
  },
  systemsettings: {
    icon: VscSettingsGear,
    brandColor: '#0078D4',
    shadowColor: 'rgba(0, 120, 212, 0.5)',
  },
  settings: {
    icon: VscSettingsGear,
    brandColor: '#0078D4',
    shadowColor: 'rgba(0, 120, 212, 0.5)',
  },
  configuracion: {
    icon: VscSettingsGear,
    brandColor: '#0078D4',
    shadowColor: 'rgba(0, 120, 212, 0.5)',
  },
  preferencias: {
    icon: VscSettingsGear,
    brandColor: '#0078D4',
    shadowColor: 'rgba(0, 120, 212, 0.5)',
  },
  kde: {
    icon: SiKde,
    brandColor: '#00C3FF',
    shadowColor: 'rgba(0, 195, 255, 0.5)',
  },
  plasma: {
    icon: SiKde,
    brandColor: '#00C3FF',
    shadowColor: 'rgba(0, 195, 255, 0.5)',
  },
  notepad: {
    icon: BsFileText,
    brandColor: '#0078D4',
    shadowColor: 'rgba(0, 120, 212, 0.5)',
  },
  blocdenotas: {
    icon: BsFileText,
    brandColor: '#0078D4',
    shadowColor: 'rgba(0, 120, 212, 0.5)',
  },
  'bloc de notas': {
    icon: BsFileText,
    brandColor: '#0078D4',
    shadowColor: 'rgba(0, 120, 212, 0.5)',
  },
  calc: {
    icon: BsCalculator,
    brandColor: '#0078D4',
    shadowColor: 'rgba(0, 120, 212, 0.5)',
  },
  calculator: {
    icon: BsCalculator,
    brandColor: '#0078D4',
    shadowColor: 'rgba(0, 120, 212, 0.5)',
  },
  calculadora: {
    icon: BsCalculator,
    brandColor: '#0078D4',
    shadowColor: 'rgba(0, 120, 212, 0.5)',
  },
  paint: {
    icon: BsPaintBucket,
    brandColor: '#0078D4',
    shadowColor: 'rgba(0, 120, 212, 0.5)',
  },
  mspaint: {
    icon: BsPaintBucket,
    brandColor: '#0078D4',
    shadowColor: 'rgba(0, 120, 212, 0.5)',
  },
};

export function getAppBrand(windowClass?: string, appName?: string, title?: string, iconKey?: string): AppBrandInfo | null {
  const candidates = [
    iconKey?.toLowerCase() || '',
    windowClass?.toLowerCase() || '',
    appName?.toLowerCase() || '',
    title?.toLowerCase() || '',
  ];

  for (const query of candidates) {
    if (!query) continue;

    for (const [key, brand] of Object.entries(BRAND_REGISTRY)) {
      if (query === key || query.includes(key) || (query.length >= 3 && key.includes(query))) {
        return brand;
      }
    }
  }

  return null;
}


export interface AppIconProps {
  iconPath?: string;
  windowClass?: string;
  appName?: string;
  title?: string;
  className?: string;
}

/**
 * Authentic Icon Component:
 * - If an icon path or data URL is provided, renders the real desktop icon using convertFileSrc.
 * - Otherwise matches registered brand SVGs.
 * - For unmapped apps without paths, renders a sleek acrylic glass tile with the initial letter.
 */
export const AppIcon: React.FC<AppIconProps> = ({
  iconPath,
  windowClass,
  appName,
  title,
  className = 'w-full h-full',
}) => {
  const [imgError, setImgError] = React.useState(false);

  React.useEffect(() => {
    setImgError(false);
  }, [iconPath, windowClass]);

  const checkIsAbs = (p?: string) => !!p && (p.startsWith('/') || /^[a-zA-Z]:[/\\]/.test(p));
  const rawPath = (iconPath || (checkIsAbs(windowClass) ? windowClass : (checkIsAbs(appName) ? appName : ''))) || '';
  const isAbsPath = checkIsAbs(rawPath);
  const isUrl = !!rawPath && /^(https?:|data:|asset:)/.test(rawPath);

  if (!imgError && (isAbsPath || isUrl)) {
    return (
      <div className="w-full h-full flex items-center justify-center p-0.5">
        <img
          src={isAbsPath ? convertFileSrc(rawPath) : rawPath}
          alt={appName || title || 'App'}
          draggable={false}
          decoding="async"
          className={`${className} object-contain select-none drop-shadow-[0_3px_8px_rgba(0,0,0,0.7)]`}
          onError={() => setImgError(true)}
        />
      </div>
    );
  }

  const brand = getAppBrand(windowClass, appName, title, iconPath || rawPath);

  if (brand) {
    const IconComponent = brand.icon;
    return (
      <div className="w-full h-full flex items-center justify-center p-1">
        <IconComponent
          className={className}
          style={{
            color: brand.brandColor,
            filter: `drop-shadow(0 4px 10px ${brand.shadowColor}) drop-shadow(0 2px 4px rgba(0,0,0,0.8))`,
          }}
        />
      </div>
    );
  }

  // Modern acrylic glass tile fallback for unmapped applications
  const displayName = title || appName || windowClass || iconPath || 'App';
  let initial = displayName.trim().slice(0, 2).toUpperCase() || 'AP';
  if (initial === 'NA' || initial === 'N/' || initial === 'UN') {
    const fallbackCandidates = [appName, title, windowClass, iconPath]
      .filter((s): s is string => !!s && !s.toLowerCase().startsWith('n/a') && !s.toLowerCase().startsWith('na'))
      .map(s => s.trim().slice(0, 2).toUpperCase())
      .filter(s => s.length >= 1);
    initial = fallbackCandidates[0] || 'AP';
  }

  return (
    <div className="w-full h-full rounded-xl bg-white/[0.06] border border-white/20 shadow-[inset_0_1px_1px_rgba(255,255,255,0.2),0_4px_12px_rgba(0,0,0,0.5)] backdrop-blur-md flex flex-col items-center justify-center p-1 select-none">
      <span className="text-[9.5px] font-mono font-bold text-white/90 uppercase tracking-tighter leading-none mb-0.5">
        {initial}
      </span>
      <svg className="w-3.5 h-3.5 text-white/60" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <rect x="3" y="3" width="18" height="18" rx="3" strokeWidth={1.8} />
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.8} d="M3 9h18M9 21V9" />
      </svg>
    </div>
  );
};

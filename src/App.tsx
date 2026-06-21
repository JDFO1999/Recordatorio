import { useEffect } from 'react';
import { useAppStore } from './store/appStore';
import { useShortcuts } from './hooks/useShortcuts';
import { Layout } from './components/Layout';
import { Dashboard } from './pages/Dashboard';
import { CreateReminder } from './pages/CreateReminder';
import { ConfirmReminder } from './pages/ConfirmReminder';
import { History } from './pages/History';
import { Settings } from './pages/Settings';
import { listen } from '@tauri-apps/api/event';
import { getSetting } from './services/settingsService';

function App() {
  const { currentView, setCurrentView, setDarkMode } = useAppStore();

  useShortcuts();

  useEffect(() => {
    const saved = localStorage.getItem('darkMode');
    if (saved !== null) {
      const isDark = saved === 'true';
      setDarkMode(isDark);
      document.documentElement.classList.toggle('dark', isDark);
    } else {
      getSetting('theme').then((val) => {
        const isDark = val === 'dark';
        setDarkMode(isDark);
        localStorage.setItem('darkMode', String(isDark));
        document.documentElement.classList.toggle('dark', isDark);
      }).catch(() => {});
    }
  }, [setDarkMode]);

  useEffect(() => {
    const unlisten = listen<string>('global-shortcut:new-reminder', () => {
      setCurrentView('create');
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [setCurrentView]);

  // Escape key: go back to dashboard from sub-pages
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && currentView !== 'dashboard' && currentView !== 'settings') {
        setCurrentView('dashboard');
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [currentView, setCurrentView]);

  const renderView = () => {
    switch (currentView) {
      case 'dashboard':
        return <Dashboard />;
      case 'create':
        return <CreateReminder />;
      case 'confirm':
        return <ConfirmReminder />;
      case 'history':
        return <History />;
      case 'settings':
        return <Settings />;
      default:
        return <Dashboard />;
    }
  };

  return (
    <Layout>
      {renderView()}
    </Layout>
  );
}

export default App;

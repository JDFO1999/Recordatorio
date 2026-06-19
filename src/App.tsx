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

function App() {
  const { currentView, setCurrentView, darkMode, setDarkMode } = useAppStore();

  useShortcuts();

  useEffect(() => {
    const savedDark = localStorage.getItem('darkMode') === 'true';
    if (savedDark) {
      setDarkMode(true);
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
  }, [setDarkMode]);

  useEffect(() => {
    if (darkMode) {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
  }, [darkMode]);

  useEffect(() => {
    const unlisten = listen<string>('global-shortcut:new-reminder', () => {
      setCurrentView('create');
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [setCurrentView]);

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

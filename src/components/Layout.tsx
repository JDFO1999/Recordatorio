import { useAppStore } from '../store/appStore';

interface LayoutProps {
  children: React.ReactNode;
}

export function Layout({ children }: LayoutProps) {
  const { currentView, setCurrentView, darkMode, toggleTheme } = useAppStore();

  const navItems = [
    { id: 'dashboard' as const, label: 'Inicio', icon: '🏠' },
    { id: 'history' as const, label: 'Historial', icon: '📋' },
    { id: 'settings' as const, label: 'Configuración', icon: '⚙️' },
  ];

  return (
    <div className="h-screen flex flex-col bg-gray-50 dark:bg-gray-900">
      <header className="relative flex items-center justify-center h-12 border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 shrink-0">
        <h1 className="text-base font-semibold text-gray-700 dark:text-gray-300 select-none">
          Recordatorio
        </h1>
        <button
          onClick={toggleTheme}
          className="absolute left-3 w-8 h-8 flex items-center justify-center rounded-full hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
          title={darkMode ? 'Modo claro' : 'Modo oscuro'}
        >
          <span className="text-base">{darkMode ? '☀️' : '🌙'}</span>
        </button>
      </header>
      <main className="flex-1 overflow-y-auto">
        {children}
      </main>
      <nav className="flex items-center justify-around bg-white dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700 py-2 px-4">
        {navItems.map((item) => (
          <button
            key={item.id}
            onClick={() => setCurrentView(item.id)}
            className={`flex flex-col items-center gap-1 px-4 py-1 rounded-lg transition-colors ${
              currentView === item.id
                ? 'text-blue-600 dark:text-blue-400'
                : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'
            }`}
          >
            <span className="text-lg">{item.icon}</span>
            <span className="text-xs">{item.label}</span>
          </button>
        ))}
      </nav>
    </div>
  );
}

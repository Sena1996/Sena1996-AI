import { Routes, Route } from 'react-router-dom';
import Layout from './components/Layout';
import { ToastProvider } from './components/Toast';
import Dashboard from './pages/Dashboard';
import Providers from './pages/Providers';
import Chat from './pages/Chat';
import Sessions from './pages/Sessions';
import Peers from './pages/Peers';
import Settings from './pages/Settings';
import Tools from './pages/Tools';
import Memory from './pages/Memory';
import Features from './pages/Features';
import Guardian from './pages/Guardian';
import Devil from './pages/Devil';

function App() {
  return (
    <ToastProvider>
      <Layout>
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/providers" element={<Providers />} />
          <Route path="/chat" element={<Chat />} />
          <Route path="/sessions" element={<Sessions />} />
          <Route path="/tools" element={<Tools />} />
          <Route path="/memory" element={<Memory />} />
          <Route path="/guardian" element={<Guardian />} />
          <Route path="/devil" element={<Devil />} />
          <Route path="/peers" element={<Peers />} />
          <Route path="/features" element={<Features />} />
          <Route path="/settings" element={<Settings />} />
        </Routes>
      </Layout>
    </ToastProvider>
  );
}

export default App;

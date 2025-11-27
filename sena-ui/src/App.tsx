import { Routes, Route } from 'react-router-dom';
import Layout from './components/Layout';
import Dashboard from './pages/Dashboard';
import Providers from './pages/Providers';
import Chat from './pages/Chat';
import Sessions from './pages/Sessions';
import Settings from './pages/Settings';

function App() {
  return (
    <Layout>
      <Routes>
        <Route path="/" element={<Dashboard />} />
        <Route path="/providers" element={<Providers />} />
        <Route path="/chat" element={<Chat />} />
        <Route path="/sessions" element={<Sessions />} />
        <Route path="/settings" element={<Settings />} />
      </Routes>
    </Layout>
  );
}

export default App;

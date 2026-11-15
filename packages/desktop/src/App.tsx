import type { FC } from 'react';
import { HashRouter, Navigate, Route, Routes, useNavigate } from 'react-router-dom';
import NotebookWorkspace from './NotebookWorkspace';
import LabPage from './components/lab/LabPage';
import ProductWelcome from './components/welcome/ProductWelcome';
import { readProductView, writeProductView, type ProductView } from './lib/product';

const ProductRedirect: FC = () => {
  const product = readProductView();
  const target = product === 'lab' ? '/lab' : product === 'notebook' ? '/notebook' : '/welcome';
  return <Navigate to={target} replace />;
};

const WelcomeRoute: FC = () => {
  const navigate = useNavigate();
  const select = (view: Exclude<ProductView, 'welcome'>) => {
    writeProductView(view);
    navigate(view === 'lab' ? '/lab' : '/notebook');
  };
  return (
    <main data-component='App' className='h-screen overflow-hidden'>
      <ProductWelcome onSelect={select} />
    </main>
  );
};

const App: FC = () => {
  return (
    <HashRouter>
      <Routes>
        <Route path='/' element={<ProductRedirect />} />
        <Route path='/welcome' element={<WelcomeRoute />} />
        <Route path='/lab/*' element={<LabPage />} />
        <Route path='/notebook' element={<NotebookWorkspace />} />
        <Route path='*' element={<Navigate to='/' replace />} />
      </Routes>
    </HashRouter>
  );
};
App.displayName = 'App';
export default App;

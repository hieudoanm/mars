import { render, screen, fireEvent } from '@testing-library/react';
import { MemoryRouter, Route, Routes, useLocation } from 'react-router-dom';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import LabPage from '../LabPage';

const mocks = vi.hoisted(() => ({
  lab: {
    selectJob: vi.fn(),
    loadCompare: vi.fn(),
  },
  view: null as Record<string, unknown> | null,
}));

vi.mock('../../../hooks/useLab', () => ({
  useLab: () => mocks.lab,
}));

vi.mock('../LabView', () => ({
  default: (props: { view: unknown; onNavigate: (view: unknown) => void; onHome?: () => void }) => {
    mocks.view = props.view as Record<string, unknown>;
    return (
      <div>
        <span data-testid='view-kind'>{(props.view as { kind: string }).kind}</span>
        <button onClick={() => props.onNavigate({ kind: 'home' })}>nav-home</button>
        <button onClick={() => props.onHome?.()}>go-home</button>
      </div>
    );
  },
}));

const LocationProbe = () => {
  const location = useLocation();
  return <div data-testid='location'>{location.pathname + location.search}</div>;
};

const renderAt = (path: string) =>
  render(
    <MemoryRouter initialEntries={[path]}>
      <LocationProbe />
      <Routes>
        <Route path='/lab/*' element={<LabPage />} />
        <Route path='/welcome' element={<div>welcome</div>} />
      </Routes>
    </MemoryRouter>,
  );

describe('LabPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.view = null;
  });

  it('maps /lab to the home view', () => {
    renderAt('/lab');
    expect(screen.getByTestId('view-kind')).toHaveTextContent('home');
  });

  it('maps /lab/new to the new-job view', () => {
    renderAt('/lab/new');
    expect(screen.getByTestId('view-kind')).toHaveTextContent('new-job');
  });

  it('maps /lab/jobs/:id to a job view and selects the job', () => {
    renderAt('/lab/jobs/job-42');
    expect(screen.getByTestId('view-kind')).toHaveTextContent('job');
    expect(mocks.view).toMatchObject({ kind: 'job', jobId: 'job-42' });
    expect(mocks.lab.selectJob).toHaveBeenCalledWith('job-42');
  });

  it('maps /lab/compare to a compare view and loads details', () => {
    renderAt('/lab/compare?ids=a,b');
    expect(screen.getByTestId('view-kind')).toHaveTextContent('compare');
    expect(mocks.view).toMatchObject({ kind: 'compare', jobIds: ['a', 'b'] });
    expect(mocks.lab.loadCompare).toHaveBeenCalledWith(['a', 'b']);
  });

  it('treats unknown paths as home', () => {
    renderAt('/lab/unknown');
    expect(screen.getByTestId('view-kind')).toHaveTextContent('home');
  });

  it('navigates when LabView requests a route change', () => {
    renderAt('/lab');
    fireEvent.click(screen.getByText('nav-home'));
    expect(screen.getByTestId('location')).toHaveTextContent('/lab');
    fireEvent.click(screen.getByText('go-home'));
    expect(screen.getByText('welcome')).toBeInTheDocument();
  });
});

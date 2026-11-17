import { useEffect, type FC } from 'react';
import { useLocation, useNavigate, useSearchParams } from 'react-router-dom';
import { useLab, type LabRouteView } from '../../hooks/useLab';
import LabView from './LabView';

const pathFor = (view: LabRouteView): string => {
  switch (view.kind) {
    case 'home':
      return '/lab';
    case 'new-job':
      return '/lab/new';
    case 'job':
      return `/lab/jobs/${view.jobId}`;
    case 'compare':
      return `/lab/compare?ids=${view.jobIds.join(',')}`;
    default:
      return '/lab';
  }
};

const viewFromLocation = (
  pathname: string,
  jobId: string | undefined,
  searchParams: URLSearchParams,
): LabRouteView => {
  if (pathname === '/lab/new') return { kind: 'new-job' };
  if (pathname.startsWith('/lab/jobs/') && jobId) return { kind: 'job', jobId };
  if (pathname === '/lab/compare') {
    const ids = (searchParams.get('ids') ?? '')
      .split(',')
      .map((id) => id.trim())
      .filter(Boolean);
    return { kind: 'compare', jobIds: ids };
  }
  return { kind: 'home' };
};

const LabPage: FC = () => {
  const lab = useLab();
  const navigate = useNavigate();
  const location = useLocation();
  const [searchParams] = useSearchParams();
  // The /lab/* splat route does not expose named params, so derive the job id
  // from the pathname directly.
  const jobId = location.pathname.startsWith('/lab/jobs/')
    ? location.pathname.slice('/lab/jobs/'.length).split('/')[0]
    : undefined;
  const view = viewFromLocation(location.pathname, jobId || undefined, searchParams);

  useEffect(() => {
    if (view.kind === 'job') void lab.selectJob(view.jobId);
    if (view.kind === 'compare' && view.jobIds.length > 0) {
      void lab.loadCompare(view.jobIds);
    }
  }, [location.pathname, location.search]);

  return (
    <LabView
      view={view}
      onNavigate={(next) => navigate(pathFor(next))}
      onHome={() => navigate('/welcome')}
    />
  );
};

LabPage.displayName = 'LabPage';
export default LabPage;

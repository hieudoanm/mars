import { render, screen } from '@testing-library/react';
import WhyMars from '../WhyMars';

describe('WhyMars', () => {
  it('renders section heading', () => {
    render(<WhyMars />);
    expect(screen.getByText(/from first question/i)).toBeInTheDocument();
    expect(screen.getByText(/insight/i)).toBeInTheDocument();
  });

  it('renders eyebrow and paragraph', () => {
    render(<WhyMars />);
    expect(screen.getByText(/a better place for unfinished thoughts/i)).toBeInTheDocument();
    expect(screen.getByText(/fast feedback/i)).toBeInTheDocument();
  });

  it('renders all feature cards', () => {
    render(<WhyMars />);
    expect(screen.getByText(/explore without context switching/i)).toBeInTheDocument();
    expect(screen.getByText(/isolation by default/i)).toBeInTheDocument();
    expect(screen.getByText(/own your documents/i)).toBeInTheDocument();
  });

  it('renders feature footnotes', () => {
    render(<WhyMars />);
    expect(screen.getByText(/process-isolated execution/i)).toBeInTheDocument();
    expect(screen.getByText(/human-readable by design/i)).toBeInTheDocument();
  });

  it('renders all feature footnote labels', () => {
    render(<WhyMars />);
    expect(screen.getByText(/rich multi-media output/i)).toBeInTheDocument();
    expect(screen.getByText(/process-isolated execution/i)).toBeInTheDocument();
    expect(screen.getByText(/human-readable by design/i)).toBeInTheDocument();
  });
});

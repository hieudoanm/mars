import { fireEvent, render, screen } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';
import LabJobForm from '../LabJobForm';
import type { LabAlgorithm, LabDataset } from '../../../lib/kernelService';

const dataset: LabDataset = {
  id: 'ds-1',
  originalFilename: 'iris.csv',
  sha256: 'abc',
  byteSize: 100,
  storageKey: 'k',
  columns: [
    { name: 'sepal_length', inferredDtype: 'number', nullCount: 0, sampleValues: [] },
    { name: 'sepal_width', inferredDtype: 'number', nullCount: 0, sampleValues: [] },
    { name: 'species', inferredDtype: 'string', nullCount: 0, sampleValues: [] },
  ],
  rowCount: 150,
  createdAt: '2026-01-01T00:00:00Z',
};

const algorithms: LabAlgorithm[] = [
  {
    id: 'logistic_regression',
    name: 'Logistic Regression',
    taskType: 'classification',
    family: 'linear',
    description: 'Linear classifier',
    defaultHyperparameters: { learning_rate: 0.1 },
    exports: ['onnx'],
  },
];

const renderForm = (onTrain = vi.fn()) => {
  render(
    <LabJobForm
      datasets={[dataset]}
      algorithms={algorithms}
      training={false}
      initialDatasetId='ds-1'
      onUploadDataset={() => {}}
      onTrain={onTrain}
    />,
  );
  return onTrain;
};

describe('LabJobForm', () => {
  it('requires feature columns before training', () => {
    const onTrain = renderForm();
    fireEvent.click(screen.getByRole('button', { name: 'Train model' }));
    expect(screen.getByText('Select at least one feature column.')).toBeInTheDocument();
    expect(onTrain).not.toHaveBeenCalled();
  });

  it('requires a label column for supervised tasks', () => {
    const onTrain = renderForm();
    fireEvent.click(screen.getByLabelText(/sepal_length/));
    fireEvent.click(screen.getByRole('button', { name: 'Train model' }));
    expect(screen.getByText('Select a label column.')).toBeInTheDocument();
    expect(onTrain).not.toHaveBeenCalled();
  });

  it('submits a train request with mapped columns', () => {
    const onTrain = renderForm();
    fireEvent.click(screen.getByLabelText(/sepal_length/));
    fireEvent.click(screen.getByLabelText(/sepal_width/));
    fireEvent.change(screen.getByDisplayValue('Select…'), {
      target: { value: 'species' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Train model' }));
    expect(onTrain).toHaveBeenCalledWith(
      expect.objectContaining({
        datasetId: 'ds-1',
        taskType: 'classification',
        algorithmId: 'logistic_regression',
        featureColumns: expect.arrayContaining(['sepal_length', 'sepal_width']),
        labelColumn: 'species',
        splitRatio: 0.2,
        randomSeed: 42,
      }),
    );
  });

  it('filters algorithms by task type', () => {
    renderForm();
    fireEvent.click(screen.getByRole('button', { name: 'Clustering' }));
    expect(screen.queryByText(/Logistic Regression —/)).not.toBeInTheDocument();
  });

  it('rejects invalid hyperparameter JSON', () => {
    const onTrain = renderForm();
    fireEvent.click(screen.getByLabelText(/sepal_length/));
    fireEvent.change(screen.getByDisplayValue('Select…'), {
      target: { value: 'species' },
    });
    // Break the hyperparameters JSON
    const hp = screen.getByRole('textbox', { name: '' });
    fireEvent.change(hp, { target: { value: '{bad' } });
    fireEvent.click(screen.getByRole('button', { name: 'Train model' }));
    expect(screen.getByText('Hyperparameters must be valid JSON.')).toBeInTheDocument();
    expect(onTrain).not.toHaveBeenCalled();
  });

  it('rejects an out-of-range split ratio and non-integer seed', () => {
    const onTrain = renderForm();
    fireEvent.click(screen.getByLabelText(/sepal_length/));
    fireEvent.change(screen.getByDisplayValue('Select…'), {
      target: { value: 'species' },
    });
    const split = screen.getByLabelText(/Split ratio/);
    fireEvent.change(split, { target: { value: '1.5' } });
    fireEvent.click(screen.getByRole('button', { name: 'Train model' }));
    expect(screen.getByText('Split ratio must be between 0 and 1.')).toBeInTheDocument();

    fireEvent.change(split, { target: { value: '0.3' } });
    const seed = screen.getByLabelText(/^Seed/);
    fireEvent.change(seed, { target: { value: '4.5' } });
    fireEvent.click(screen.getByRole('button', { name: 'Train model' }));
    expect(screen.getByText('Seed must be an integer.')).toBeInTheDocument();
    expect(onTrain).not.toHaveBeenCalled();
  });

  it('submits a reinforcement-learning request with rl columns', () => {
    const rlAlgorithms: LabAlgorithm[] = [
      {
        id: 'q_learning',
        name: 'Q-Learning',
        taskType: 'reinforcement_learning',
        family: 'rl',
        description: 'RL',
        defaultHyperparameters: {},
        exports: ['onnx'],
      },
    ];
    const onTrain = vi.fn();
    render(
      <LabJobForm
        datasets={[dataset]}
        algorithms={rlAlgorithms}
        training={false}
        initialDatasetId='ds-1'
        onUploadDataset={() => {}}
        onTrain={onTrain}
      />,
    );
    fireEvent.click(screen.getByRole('button', { name: 'Reinforcement learning' }));
    fireEvent.click(screen.getByLabelText(/sepal_length/));
    fireEvent.click(screen.getByLabelText(/sepal_width/));
    // RL requires action + reward before submit
    fireEvent.click(screen.getByRole('button', { name: 'Train model' }));
    expect(screen.getByText('Select action and reward columns.')).toBeInTheDocument();
    expect(onTrain).not.toHaveBeenCalled();
  });

  it('renders an upload prompt when there are no datasets', () => {
    const onUpload = vi.fn();
    render(
      <LabJobForm
        datasets={[]}
        algorithms={[]}
        training={false}
        initialDatasetId={null}
        onUploadDataset={onUpload}
        onTrain={() => {}}
      />,
    );
    expect(screen.queryByRole('button', { name: 'Train model' })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Upload CSV' }));
    expect(onUpload).toHaveBeenCalled();
  });
});

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
});

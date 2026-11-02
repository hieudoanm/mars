import { repositoryUrl, siteUrl } from './site';

describe('site config', () => {
  it('exports repositoryUrl', () => {
    expect(repositoryUrl).toBe('https://github.com/hieudoanm/mars');
  });

  it('exports siteUrl', () => {
    expect(typeof siteUrl).toBe('string');
    expect(siteUrl).toBeTruthy();
  });
});

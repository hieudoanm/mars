import { csvSnippet } from '../cellCsv';

describe('csvSnippet', () => {
  it('wraps CSV-looking text in mars_csv', () => {
    expect(csvSnippet('name,score\nalice,10\nbob,20')).toBe(
      'mars_csv(r#"name,score\nalice,10\nbob,20"#);',
    );
  });

  it('rejects single-line and non-tabular text', () => {
    expect(csvSnippet('just,one,line')).toBeNull();
    expect(csvSnippet('no commas here\nsecond line')).toBeNull();
    expect(csvSnippet('a,b,c\n1,2')).toBe('mars_csv(r#"a,b,c\n1,2"#);');
  });

  it('picks a raw-string delimiter that cannot collide', () => {
    const text = 'a,b\n"#quoted"#,2';
    const snippet = csvSnippet(text);
    expect(snippet).toContain('r##"');
    expect(snippet).toContain('"##');
  });
});

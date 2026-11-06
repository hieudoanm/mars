export type ClassName = string | number | false | null | undefined;

/**
 * Joins conditional class names while dropping empty values.
 *
 * Kept intentionally small so components do not need a utility dependency.
 */
export const cn = (...values: ClassName[]): string => {
  const classes: string[] = [];

  for (const value of values) {
    if (value === false || value === null || value === undefined || value === '') continue;
    classes.push(String(value));
  }

  return classes.join(' ');
};

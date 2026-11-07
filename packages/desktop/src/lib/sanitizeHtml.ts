import DOMPurify from 'dompurify';

export const sanitizeHtml = (html: string): string => {
  return DOMPurify.sanitize(html, {
    USE_PROFILES: { html: true, svg: true, svgFilters: true },
    FORBID_TAGS: ['form', 'foreignObject', 'iframe', 'math', 'object', 'script', 'template'],
    FORBID_ATTR: ['formaction', 'srcset', 'style'],
    ALLOW_DATA_ATTR: false,
  });
};

import type { InputHTMLAttributes, TextareaHTMLAttributes } from 'react';

export function TextField({
  label,
  hint,
  ...props
}: InputHTMLAttributes<HTMLInputElement> & { label: string; hint?: string }) {
  return (
    <label className="field">
      <span className="field__label">{label}</span>
      <input className="field__control" {...props} />
      {hint ? <span className="field__hint">{hint}</span> : null}
    </label>
  );
}

export function TextAreaField({
  label,
  hint,
  ...props
}: TextareaHTMLAttributes<HTMLTextAreaElement> & { label: string; hint?: string }) {
  return (
    <label className="field">
      <span className="field__label">{label}</span>
      <textarea className="field__control field__textarea" {...props} />
      {hint ? <span className="field__hint">{hint}</span> : null}
    </label>
  );
}

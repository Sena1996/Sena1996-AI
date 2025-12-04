import { useState } from 'react';
import { Eye, EyeOff, HelpCircle } from 'lucide-react';
import type { AuthField } from '../types';

interface DynamicAuthFormProps {
  fields: AuthField[];
  values: Record<string, string>;
  onChange: (fieldId: string, value: string) => void;
  errors: Record<string, string>;
  disabled?: boolean;
}

export default function DynamicAuthForm({
  fields,
  values,
  onChange,
  errors,
  disabled = false,
}: DynamicAuthFormProps) {
  return (
    <div className="space-y-4">
      {fields.map((field) => (
        <FieldInput
          key={field.id}
          field={field}
          value={values[field.id] || field.defaultValue || ''}
          onChange={(value) => onChange(field.id, value)}
          error={errors[field.id]}
          disabled={disabled}
        />
      ))}
    </div>
  );
}

function FieldInput({
  field,
  value,
  onChange,
  error,
  disabled,
}: {
  field: AuthField;
  value: string;
  onChange: (value: string) => void;
  error?: string;
  disabled: boolean;
}) {
  const [showPassword, setShowPassword] = useState(false);

  const inputType = field.fieldType === 'password' && !showPassword
    ? 'password'
    : field.fieldType === 'number'
      ? 'number'
      : 'text';

  const validateInput = (inputValue: string) => {
    if (field.validationPattern) {
      const regex = new RegExp(field.validationPattern);
      return regex.test(inputValue);
    }
    return true;
  };

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onChange(e.target.value);
  };

  if (field.fieldType === 'toggle') {
    return (
      <div className="flex items-center justify-between">
        <div>
          <label className="text-sm font-medium text-dark-200">
            {field.displayName}
            {field.required && <span className="text-red-400 ml-1">*</span>}
          </label>
          {field.helpText && (
            <p className="text-xs text-dark-400 mt-0.5">{field.helpText}</p>
          )}
        </div>
        <button
          type="button"
          onClick={() => onChange(value === 'true' ? 'false' : 'true')}
          disabled={disabled}
          className={`relative w-11 h-6 rounded-full transition-colors ${
            value === 'true' ? 'bg-sena-500' : 'bg-dark-700'
          } ${disabled ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer'}`}
        >
          <span
            className={`absolute top-0.5 left-0.5 w-5 h-5 bg-white rounded-full transition-transform ${
              value === 'true' ? 'translate-x-5' : 'translate-x-0'
            }`}
          />
        </button>
      </div>
    );
  }

  return (
    <div>
      <div className="flex items-center gap-2 mb-1.5">
        <label htmlFor={field.id} className="text-sm font-medium text-dark-200">
          {field.displayName}
          {field.required && <span className="text-red-400 ml-1">*</span>}
        </label>
        {field.helpText && (
          <div className="group relative">
            <HelpCircle className="w-4 h-4 text-dark-500 cursor-help" />
            <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-3 py-2 bg-dark-700 text-xs text-dark-200 rounded-lg opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all whitespace-nowrap z-10 shadow-lg">
              {field.helpText}
              <div className="absolute top-full left-1/2 -translate-x-1/2 -mt-1 border-4 border-transparent border-t-dark-700" />
            </div>
          </div>
        )}
      </div>

      <div className="relative">
        <input
          id={field.id}
          type={inputType}
          value={value}
          onChange={handleChange}
          placeholder={field.placeholder || undefined}
          disabled={disabled}
          className={`w-full px-4 py-2.5 bg-dark-800 border rounded-lg text-dark-100 placeholder-dark-500 focus:outline-none focus:ring-2 focus:ring-sena-500/50 focus:border-sena-500 transition-colors ${
            error
              ? 'border-red-500/50'
              : value && !validateInput(value)
                ? 'border-yellow-500/50'
                : 'border-dark-700'
          } ${disabled ? 'opacity-50 cursor-not-allowed' : ''} ${
            field.sensitive ? 'pr-10 font-mono' : ''
          }`}
        />

        {field.fieldType === 'password' && (
          <button
            type="button"
            onClick={() => setShowPassword(!showPassword)}
            disabled={disabled}
            className="absolute right-3 top-1/2 -translate-y-1/2 text-dark-400 hover:text-dark-200 transition-colors"
          >
            {showPassword ? (
              <EyeOff className="w-4 h-4" />
            ) : (
              <Eye className="w-4 h-4" />
            )}
          </button>
        )}
      </div>

      {error && (
        <p className="text-xs text-red-400 mt-1.5">{error}</p>
      )}

      {field.envVarName && !error && (
        <p className="text-xs text-dark-500 mt-1.5">
          Environment: <code className="text-dark-400">{field.envVarName}</code>
        </p>
      )}
    </div>
  );
}

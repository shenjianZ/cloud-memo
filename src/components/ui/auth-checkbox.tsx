import { Check } from 'lucide-react'

interface AuthCheckboxProps {
  checked: boolean
  onCheckedChange: (checked: boolean) => void
  disabled?: boolean
  id?: string
}

/**
 * Auth 页面专用 Checkbox 组件
 * 使用内联样式确保对钩正确显示
 */
export function AuthCheckbox({ checked, onCheckedChange, disabled, id }: AuthCheckboxProps) {
  return (
    <button
      type="button"
      role="checkbox"
      aria-checked={checked}
      id={id}
      disabled={disabled}
      onClick={() => !disabled && onCheckedChange(!checked)}
      style={{
        width: '16px',
        height: '16px',
        minWidth: '16px',
        minHeight: '16px',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        borderRadius: '4px',
        border: '1px solid #3b82f6',
        backgroundColor: checked ? '#3b82f6' : 'white',
        cursor: disabled ? 'not-allowed' : 'pointer',
        opacity: disabled ? 0.5 : 1,
        padding: '0',
        transition: 'all 0.2s',
        boxSizing: 'border-box',
      }}
    >
      {checked && (
        <Check
          style={{
            width: '10px',
            height: '10px',
            color: '#ffffff',
            strokeWidth: 3,
            display: 'block',
          }}
        />
      )}
    </button>
  )
}

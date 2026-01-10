import { ButtonHTMLAttributes, createContext, useContext, forwardRef, ReactNode } from 'react';
import styles from './Tabs.module.css';

interface TabsContextValue {
  value: string;
  onChange: (value: string) => void;
}

const TabsContext = createContext<TabsContextValue | null>(null);

export interface TabsProps {
  value: string;
  onChange: (value: string) => void;
  children: ReactNode;
  className?: string;
}

export interface TabProps extends Omit<ButtonHTMLAttributes<HTMLButtonElement>, 'value' | 'onChange'> {
  value: string;
  children: ReactNode;
}

export const Tabs = ({ value, onChange, children, className = '' }: TabsProps) => {
  const classes = [styles.tabs, className].filter(Boolean).join(' ');

  return (
    <TabsContext.Provider value={{ value, onChange }}>
      <div className={classes} role="tablist">
        {children}
      </div>
    </TabsContext.Provider>
  );
};

const Tab = forwardRef<HTMLButtonElement, TabProps>(
  ({ value, children, className = '', ...props }, ref) => {
    const context = useContext(TabsContext);
    if (!context) {
      throw new Error('Tabs.Tab must be used within a Tabs component');
    }

    const isActive = context.value === value;
    const classes = [
      styles.tab,
      isActive && styles.active,
      className,
    ].filter(Boolean).join(' ');

    return (
      <button
        ref={ref}
        type="button"
        role="tab"
        aria-selected={isActive}
        className={classes}
        onClick={() => context.onChange(value)}
        {...props}
      >
        {children}
      </button>
    );
  }
);

Tab.displayName = 'Tabs.Tab';

Tabs.Tab = Tab;
Tabs.displayName = 'Tabs';

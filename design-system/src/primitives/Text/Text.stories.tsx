import '../../tokens/index.css';
import { Text } from './Text';

export default {
  title: 'Primitives/Text',
};

export const Intensities = () => (
  <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', padding: '2rem', backgroundColor: 'var(--color-bg-base)' }}>
    <Text intensity="high">High intensity - Primary content</Text>
    <Text intensity="normal">Normal intensity - Secondary content</Text>
    <Text intensity="dim">Dim intensity - Metadata and timestamps</Text>
  </div>
);

export const Monospace = () => (
  <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', padding: '2rem', backgroundColor: 'var(--color-bg-base)' }}>
    <Text>Sans-serif text (default)</Text>
    <Text mono>Monospace text (code, IDs, values)</Text>
    <Text mono intensity="dim">session_id: abc123</Text>
  </div>
);

export const Sizes = () => (
  <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', padding: '2rem', backgroundColor: 'var(--color-bg-base)' }}>
    <Text size="2xl" intensity="high">2XL Heading</Text>
    <Text size="xl" intensity="high">XL Heading</Text>
    <Text size="lg" intensity="high">LG Heading</Text>
    <Text size="base">Base text</Text>
    <Text size="sm">Small text</Text>
    <Text size="xs" intensity="dim">Extra small text</Text>
  </div>
);

export const SemanticElements = () => (
  <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', padding: '2rem', backgroundColor: 'var(--color-bg-base)' }}>
    <Text as="h1" size="2xl" intensity="high">H1 Heading</Text>
    <Text as="h2" size="xl" intensity="high">H2 Heading</Text>
    <Text as="p">Paragraph text renders as a p element</Text>
    <Text as="label" intensity="dim">Label element</Text>
  </div>
);

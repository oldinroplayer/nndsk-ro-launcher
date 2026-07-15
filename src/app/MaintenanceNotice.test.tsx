// @vitest-environment jsdom

import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import { MaintenanceNotice } from './MaintenanceNotice'

describe('MaintenanceNotice', () => {
  it('shows successful maintenance without blocking and can be dismissed', () => {
    const onDismiss = vi.fn()
    render(
      <MaintenanceNotice
        notices={[
          {
            source: 'servers',
            kind: 'recovered',
            message: 'Servidores recuperados desde el backup',
          },
        ]}
        onDismiss={onDismiss}
      />,
    )

    expect(screen.getByRole('status')).toHaveTextContent(
      'Servidores recuperados desde el backup',
    )
    fireEvent.click(screen.getByRole('button', { name: 'Descartar aviso' }))
    expect(onDismiss).toHaveBeenCalledOnce()
  })
})

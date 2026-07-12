import { type RefObject, useLayoutEffect, useRef } from 'react'

const TRANSITION_MS = 380
const TRANSITION_EASING = 'cubic-bezier(0.25, 1, 0.5, 1)'

/**
 * Compensa con transform el salto horizontal que ocurre al cambiar el ancho
 * del rail. El grid se recalcula una sola vez; los frames intermedios quedan
 * a cargo del compositor mediante transform y opacity.
 */
export function useTransformLayoutTransition<T extends HTMLElement>(
  expanded: boolean,
  targetRef: RefObject<T>,
  widthDelta: number,
) {
  const previousExpanded = useRef(expanded)

  useLayoutEffect(() => {
    const wasExpanded = previousExpanded.current
    previousExpanded.current = expanded

    if (wasExpanded === expanded) return
    if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) return

    const target = targetRef.current
    if (!target?.animate) return

    const offset = expanded ? -widthDelta : widthDelta
    const animation = target.animate(
      [
        { transform: `translateX(${offset}px)`, opacity: 0.88 },
        { transform: 'translateX(0)', opacity: 1 },
      ],
      {
        duration: TRANSITION_MS,
        easing: TRANSITION_EASING,
        fill: 'both',
      },
    )

    animation.onfinish = () => animation.cancel()
    return () => animation.cancel()
  }, [expanded, targetRef, widthDelta])
}

#include "wineventinterceptor.h"

#define WANR_CLIENT_NOT_SET \
    if (!m_client) { \
        qmlWarning(this) << "Client not set"; \
        return false; \
    }

WinEventInterceptor::WinEventInterceptor(QObject *parent)
    : QObject{parent}
{
    reassignObservation(nullptr, target());
}

bool WinEventInterceptor::eventFilter(QObject *watched, QEvent *event)
{
    if (const auto closeEvent = dynamic_cast<QCloseEvent *>(event)) {
        WANR_CLIENT_NOT_SET
        m_client->sendCloseEvent();
        return true;
    } else {
        return false;
    }
}

void WinEventInterceptor::reassignObservation(QQuickWindow *prev, QQuickWindow *next)
{
    if (prev)
        prev->removeEventFilter(this);
    if (next)
        next->installEventFilter(this);
}

void WinEventInterceptor::setTarget(QQuickWindow *newTarget)
{
    if (m_target == newTarget)
        return;

    reassignObservation(m_target, newTarget);
    m_target = newTarget;
    emit targetChanged();
}

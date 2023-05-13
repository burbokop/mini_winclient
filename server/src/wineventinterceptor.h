#pragma once

#include <QQuickWindow>
#include "winclient.h"

class QQuickWindow;
class WinEventInterceptor : public QObject
{
    Q_OBJECT
    QML_ELEMENT

    Q_PROPERTY(QQuickWindow *target READ target WRITE setTarget NOTIFY targetChanged)
    Q_PROPERTY(WinClient *client MEMBER m_client NOTIFY clientChanged)
public:
    explicit WinEventInterceptor(QObject *parent = nullptr);

    QQuickWindow *target() const { return m_target; }
    void setTarget(QQuickWindow *newTarget);

signals:
    void targetChanged();
    void clientChanged();

    // QObject interface

public:
    bool eventFilter(QObject *watched, QEvent *event) override;

private:
    void reassignObservation(QQuickWindow *prev, QQuickWindow *next);
    QQuickWindow *m_target = nullptr;
    WinClient *m_client = nullptr;
};
